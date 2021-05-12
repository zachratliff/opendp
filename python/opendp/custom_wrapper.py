
from opendp import lib
import ctypes


# ~~~~~~~ other lib code
from mod import py_to_object, FfiMeasurement, FfiTransformation


class H(object):
    def __init__(self, v, hint=None):
        self.v = v
        self.hint = hint


def infer_type_hint(v):
    return standardize_type_hint(type(v))

def standardize_type_hint(v):
    if isinstance(v, type):
        return # lookup table for python types -> rust strings
    return v

def to_c(v, type_hint):
    # if not type_hint:
    #     type_hint = infer_type_hint(v)

    if type_hint == 'u32':
        return ctypes.c_uint32(v)
    # repeat for each type

def handle_result(result):
    if isinstance(result, ctypes.POINTER(FfiResult)):
        pass
    return result

# ~~~~~~~

# 1. partition type_args into mandatory and optional
# 2. build python signature- mandatory type args are explicit args, not kwargs
# 3. generate code for each type arg
#       1. mandatory args just need to be standardized
#       2. optional args are inferred from explicit args if not defined in optional_type_args
# 4. piece together type_args from bootstrap['type_args'] list
# 5. load each arg into c representation using types from step 3
# 6. build function name
# 7. call function!
# 8. handle result

# what are we missing?
# no type arg inference
# no arg inference
#   THIS IS FINE


# 1.
# completely in preprocess code
# 2.
def make_base_laplace(sigma, **optional_type_args):
    # 3.2
    T = standardize_type_hint(optional_type_args.get('T', type(sigma)))
    # 4.
    type_args = f"<{T}>".encode()
    # 5.
    sigma = ctypes.by_ref(to_c(sigma, T))
    # 6.
    # completely in preprocess code
    # 7.
    result = lib.opendp_meas__make_base_laplace(type_args, sigma)
    # 8.
    return handle_result(result)


make_base_laplace(1.)

make_base_laplace(1)
make_base_laplace(1, T="f64")


def make_bounded_covariance(lower, upper, sigma, MI, T):
    MI = standardize_type_hint(MI)
    T = standardize_type_hint(T)  # knowing to retrieve type(lower[0]) is difficult to specify in bootstrap json
    type_args = f"<{MI},L1Sensitivity<{T}>>".encode()

    lower = py_to_object(lower, type_arg=f"({T}, {T})")
    upper = py_to_object(upper, type_arg=f"({T}, {T})")
    sigma = ctypes.by_ref(to_c(sigma, T))

    return handle_result(lib.opendp_meas__make_bounded_covariance(type_args, lower, upper, sigma))


make_bounded_covariance((1., 2.), (10., 20.), 2.)
# def make_base_stability(type_args, n, scale, threshold):
#     return
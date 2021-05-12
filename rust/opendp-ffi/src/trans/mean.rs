use std::iter::Sum;
use std::ops::Sub;
use std::os::raw::{c_char, c_uint, c_void};

use num::Float;

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::err;
use opendp::traits::DistanceConstant;
use opendp::trans::{BoundedMeanConstant};
use opendp::trans;

use crate::core::{FfiResult, FfiTransformation};
use crate::util::{Type, parse_type_args, into_raw};

#[no_mangle]
pub extern "C" fn make_bounded_mean(type_args: *const c_char, lower: *const c_void, upper: *const c_void, length: c_uint) -> *mut FfiResult<*mut FfiTransformation> {
    fn monomorphize<T>(type_args: Vec<Type>, lower: *const c_void, upper: *const c_void, length: usize) -> FfiResult<*mut FfiTransformation>
        where T: DistanceConstant + Sub<Output=T> + Float,
              for<'a> T: Sum<&'a T> {
        fn monomorphize2<MI, MO>(lower: MO::Distance, upper: MO::Distance, length: usize) -> FfiResult<*mut FfiTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  MO::Distance: DistanceConstant + Sub<Output=MO::Distance> + Float,
                  for<'a> MO::Distance: Sum<&'a MO::Distance>,
                  (MI, MO): BoundedMeanConstant<MI, MO> {
            trans::make_bounded_mean::<MI, MO>(lower, upper, length).into()
        }
        let lower = *try_as_ref!(lower as *const T);
        let upper = *try_as_ref!(upper as *const T);
        dispatch!(monomorphize2, [
            (type_args[0], [HammingDistance, SymmetricDistance]),
            (type_args[1], [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper, length))
    }
    let length = length as usize;
    let type_args = try_raw!(parse_type_args(type_args, 2));
    let type_output = try_raw!(type_args[1].get_sensitivity_distance());
    into_raw(dispatch!(monomorphize, [(type_output, @floats)], (type_args, lower, upper, length)))
}
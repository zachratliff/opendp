use std::os::raw::{c_char, c_void};

use num::Float;

use opendp::err;
use opendp::meas;
use opendp::samplers::SampleLaplace;
use opendp::traits::DistanceCast;

use crate::core::{FfiMeasurement, FfiResult};
use crate::util::{parse_type_args, into_raw};

#[no_mangle]
pub extern "C" fn make_base_laplace2(type_args: *const c_char, scale: *const c_void) -> *mut FfiResult<*mut FfiMeasurement> {
    fn monomorphize<T>(scale: *const c_void) -> FfiResult<*mut FfiMeasurement>
        where T: 'static + Clone + SampleLaplace + Float + DistanceCast {
        let scale = *try_as_ref!(scale as *const T);
        meas::make_base_laplace::<T>(scale).into()
    }
    let type_args = try_raw!(parse_type_args(type_args, 1));
    into_raw(dispatch!(monomorphize, [(type_args[0], @floats)], (scale)))
}

#[no_mangle]
pub extern "C" fn make_base_laplace_vec(type_args: *const c_char, scale: *const c_void) -> *mut FfiResult<*mut FfiMeasurement> {
    fn monomorphize<T>(scale: *const c_void) -> FfiResult<*mut FfiMeasurement>
        where T: 'static + Clone + SampleLaplace + Float + DistanceCast {
        let scale = *try_as_ref!(scale as *const T);
        meas::make_base_laplace_vec::<T>(scale).into()
    }
    let type_args = try_raw!(parse_type_args(type_args, 1));
    into_raw(dispatch!(monomorphize, [(type_args[0], @floats)], (scale)))
}

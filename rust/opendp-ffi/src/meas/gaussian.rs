use std::os::raw::{c_char, c_void};

use num::Float;

use opendp::err;
use opendp::meas;
use opendp::samplers::SampleGaussian;

use crate::core::{FfiMeasurement, FfiResult};
use crate::util::parse_type_args;
use crate::util;

#[no_mangle]
pub extern "C" fn make_base_gaussian(type_args: *const c_char, scale: *const c_void) -> *mut FfiResult<*mut FfiMeasurement> {
    fn monomorphize<T>(scale: *const c_void) -> FfiResult<*mut FfiMeasurement> where
        T: 'static + Clone + SampleGaussian + Float {
        let scale = *try_as_ref!(scale as *const T);
        meas::make_base_gaussian::<T>(scale).into()
    }
    let type_args = try_raw!(parse_type_args(type_args, 1));
    util::into_raw(dispatch!(monomorphize, [(type_args[0], @floats)], (scale)))
}

#[no_mangle]
pub extern "C" fn make_base_gaussian_vec(type_args: *const c_char, scale: *const c_void) -> *mut FfiResult<*mut FfiMeasurement> {
    fn monomorphize<T>(scale: *const c_void) -> FfiResult<*mut FfiMeasurement> where
        T: 'static + Clone + SampleGaussian + Float {
        let scale = *try_as_ref!(scale as *const T);
        meas::make_base_gaussian_vec::<T>(scale).into()
    }
    let type_args = try_raw!(parse_type_args(type_args, 1));
    util::into_raw(dispatch!(monomorphize, [(type_args[0], @floats)], (scale)))
}

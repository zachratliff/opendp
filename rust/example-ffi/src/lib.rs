

use opendp::meas::geometric::make_base_geometric;
use opendp::dom::AllDomain;

// lightweight crate to expose the geometric mechanism with an extremely simple api
// the only data left in the .so should be the parts of core that this function uses
#[no_mangle]
extern "C" fn geometric_mechanism(scale: f64, arg: i32) -> i32 {
    // add any computation here.
    let meas = make_base_geometric::<AllDomain<_>, _>(scale, None)
        .expect("failed to create the geometric measurement");

    let result = meas.function.eval(&arg)
        .expect("failed to evaluate the function");

    result
}
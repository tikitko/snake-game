use std::os::raw::{c_int, c_uint};
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

extern "C" {
    fn srand(seed: c_uint);
    fn rand() -> c_uint;
}

pub mod direction;
pub mod node;
pub mod point;
pub mod world;

pub unsafe fn get_rand_in_range(a: c_int, b: c_int) -> c_int {
    let m = (b - a + 1) as c_uint;
    a + (rand() % m) as c_int
}

pub unsafe fn set_rand_current_time_seed() -> Result<(), SystemTimeError> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
    srand(nanos);
    Ok(())
}

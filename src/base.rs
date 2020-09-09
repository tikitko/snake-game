use std::os::raw::c_uint;
use std::time::{UNIX_EPOCH, SystemTime};

pub mod direction;
pub mod node;
pub mod point;
pub mod world;

extern "C" {
    fn srand(seed: c_uint);
    fn rand() -> c_uint;
}

pub unsafe fn get_rand_in_range(a: i32, b: i32) -> i32 {
    let m = (b - a + 1) as u32;
    a + (rand() % m) as i32
}

pub unsafe fn set_rand_current_time_seed() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    srand(nanos);
}
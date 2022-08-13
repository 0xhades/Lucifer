use std::os::raw::{c_int, c_void};

#[link(name = "msvcrt")]
extern "C" {
    fn _getmaxstdio() -> c_int;
    fn _setmaxstdio(new_max: c_int) -> c_int;
}

pub const MAX_FD: i32 = 2 << 12;
const LESS_FD: i32 = 2 << 10;

pub fn raise_fd_limit(mut max: i32) -> Result<i32, i32> {
    unsafe {
        let mut current = _getmaxstdio();
        if current == MAX_FD {
            return Ok(current);
        }

        let result = _setmaxstdio(max.into());
        if result == -1 {
            max = LESS_FD;
            let result = _setmaxstdio(max.into());
            if result == -1 {
                return Err(-1);
            }
        }

        current = _getmaxstdio();
        if current == MAX_FD || current == LESS_FD {
            return Ok(current);
        } else {
            return Err(current);
        }
    }
}

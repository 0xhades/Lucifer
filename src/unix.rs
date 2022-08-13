use libc;

pub const MAX_FD: i32 = 2 << 14;
const LESS_FD: i32 = 2 << 13;

pub fn raise_fd_limit(max: i32) {}

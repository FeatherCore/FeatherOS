//! Thin platform hooks used by Wing shell systems.

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

const MAX_BUILTIN_COMMAND_LEN: usize = 63;
const WNOHANG: c_int = 1 << 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchError {
    EmptyCommand,
    CommandTooLong,
    InteriorNul,
    ExecFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Running,
    Exited,
}

pub fn launch_nuttx_builtin(command: &str) -> Result<i32, LaunchError> {
    let bytes = command.as_bytes();
    if bytes.is_empty() {
        return Err(LaunchError::EmptyCommand);
    }
    if bytes.len() > MAX_BUILTIN_COMMAND_LEN {
        return Err(LaunchError::CommandTooLong);
    }

    let mut command_buf = [0u8; MAX_BUILTIN_COMMAND_LEN + 1];
    for (index, byte) in bytes.iter().copied().enumerate() {
        if byte == 0 {
            return Err(LaunchError::InteriorNul);
        }
        command_buf[index] = byte;
    }

    let appname = command_buf.as_ptr().cast::<c_char>();
    let argv = [
        command_buf.as_mut_ptr().cast::<c_char>(),
        ptr::null_mut::<c_char>(),
    ];

    let pid = unsafe { exec_builtin(appname, argv.as_ptr(), ptr::null()) };
    if pid < 0 {
        Err(LaunchError::ExecFailed)
    } else {
        Ok(pid as i32)
    }
}

pub fn poll_nuttx_task(pid: i32) -> TaskStatus {
    if pid <= 0 {
        return TaskStatus::Exited;
    }

    let mut status = 0;
    let ret = unsafe { waitpid(pid as c_int, &mut status, WNOHANG) };
    if ret == 0 {
        TaskStatus::Running
    } else {
        TaskStatus::Exited
    }
}

unsafe extern "C" {
    fn exec_builtin(
        appname: *const c_char,
        argv: *const *mut c_char,
        param: *const c_void,
    ) -> c_int;

    fn waitpid(pid: c_int, stat_loc: *mut c_int, options: c_int) -> c_int;
}

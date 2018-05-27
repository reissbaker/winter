use std::{
    env,
    ptr,
    ffi::CString,
};

use libc;

pub fn exec() {
    // Get the desired shell
    let shell = match env::var("SHELL") {
        Ok(val) => val,
        Err(_) => "/bin/bash".to_string(),
    };

    /*
     * Exec it as a login shell
     *
     * Traditionally, shells will check if their arg0 (that is, the name
     * supposedly that they've been launched with) is prefixed with a -.
     * If so, they're launched as a login shell; otherwise, not.
     *
     * Rust doesn't expose an execl-style API, where you're able to set
     * arg0 of the exec-ed process (aka, rename it). Instead we have to
     * use the libc crate to do C FFI to get native execl support.
     */
    let shell_cstr = CString::new(shell.clone()).ok().unwrap();

    let shell_name = shell.rsplit("/").next().unwrap();
    let arg0_cstr = CString::new(format!("-{}", shell_name)).ok().unwrap();

    // execl requires its final argument to be null
    let null_ptr: *const i8 = ptr::null();

    unsafe {
        libc::execl(shell_cstr.as_ptr(), arg0_cstr.as_ptr(), null_ptr);
    }

    // If this executes, execl failed.
    println!("Failed to launch {}", shell);
}

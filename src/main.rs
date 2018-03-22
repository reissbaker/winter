extern crate libc;
extern crate pty;
extern crate termion;

use std::thread;
use std::env;
use std::io::{self, Read, Write};
use std::ffi::CString;
use std::ptr;
use pty::fork::{Fork, Master};

const CHUNK_SIZE: usize = 1024;

fn main() {
    let fork = Fork::from_ptmx().unwrap();

    match fork.is_parent() {
        // We are the master
        Ok(mut master) => {
            let mut master_clone = master.clone();
            thread::spawn(move|| {
                write_master_forever(&mut master_clone);
            });

            match read_master_forever(&mut master) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error: {:?}", e);
                },
            }
        },

        // We are the slave
        Err(_) => {
            // Get the desired shell
            let mut shell = match env::var("SHELL") {
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
            let mut null_ptr: *const i8 = ptr::null();

            unsafe {
                libc::execl(shell_cstr.as_ptr(), arg0_cstr.as_ptr(), null_ptr);
            }

            // If this executes, execl failed.
            println!("Failed to launch {}", shell);
        }
    }
}

fn read_master_forever(master: &mut Master) -> Result<(), io::Error> {
    let mut bytes: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];

    loop {
        // Get bytes from master, print to screen
        match master.read(&mut bytes) {
            // Ignore errors; if there are no more bytes coming, it will
            // return Ok(0)
            Err(_) => (),

            // Ok(0) signifies no more bytes are coming; we're at the
            // end of the stream.
            Ok(0) => break,

            // If we got some bytes, process them.
            Ok(n) => {
                let read_bytes: Vec<u8> = bytes.into_iter()
                    .take(n)
                    .map(|byte_addr| *byte_addr)
                    .collect();

                try!(io::stdout().write_all(&read_bytes));
                try!(io::stdout().flush());
            },
        }
    }

    Ok(())
}

fn write_master_forever(master: &mut Master) -> Result<(), io::Error> {
    let mut bytes: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];

    loop {
        // Get bytes from stdin, send to master
        match io::stdin().read(&mut bytes) {
            Err(_) => (),
            Ok(0) => (),
            Ok(n) => {
                let read_bytes: Vec<u8> = bytes.into_iter()
                    .take(n)
                    .map(|byte_addr| *byte_addr)
                    .collect();

                try!(master.write_all(&read_bytes));
            },
        }
    }
}

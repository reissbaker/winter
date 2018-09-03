use std::{
    thread,
    io::{self, Read, Write, Stdout},
    os::unix::io::{RawFd, AsRawFd},
    sync::{
      Arc,
      atomic::{AtomicBool, Ordering},
    },
};

use pty::fork::{Fork, Master};
use termion::raw::{IntoRawMode, RawTerminal};
use signal_hook;
use libc;

use shell;
use fd_winsize;

const CHUNK_SIZE: usize = 1024;

pub fn fork() {
    let fork = Fork::from_ptmx().unwrap();

    match fork.is_parent() {
        // We are the master
        Ok(mut master) => {
            let mut master_clone = master.clone();
            thread::spawn(move|| {
                write_master_forever(&mut master_clone);
            });

            let mut stdout_raw = io::stdout().into_raw_mode().unwrap();
            match read_master_forever(&mut master, &mut stdout_raw) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error: {:?}", e);
                },
            }
        },

        // We are the slave; exec a shell
        Err(_) => {
            shell::exec()
        }
    }
}

fn read_master_forever(master: &mut Master, stdout: &mut RawTerminal<Stdout>) -> Result<(), io::Error> {
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

                try!(stdout.write_all(&read_bytes));
                try!(stdout.flush());
            },
        }
    }

    Ok(())
}

fn write_master_forever(master: &mut Master) -> Result<(), io::Error> {
    // Register a flag that gets set to true to track when our PTY window size
    // changes. POSIX systems will send a SIGWINCH signal when that happens
    let resize = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(libc::SIGWINCH, Arc::clone(&resize));

    // Initially just reset the master PTY's window size so that it matches ours
    reset_pty_winsize(master.as_raw_fd());

    let mut bytes: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];

    loop {
        // If we've seen our PTY window size change, reset the master's
        if resize.load(Ordering::Relaxed) {
            resize.store(false, Ordering::Relaxed);
            reset_pty_winsize(master.as_raw_fd());
        }

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

fn reset_pty_winsize(fd: RawFd) {
    // If we successfully read a window size, set it on the PTY. If not,
    // whatever; the caller obv didn't care about the size
    fd_winsize::get(io::stdout().as_raw_fd()).and_then(|size| {
        fd_winsize::set(fd, size.rows, size.cols);
    });
}

use std::{
    thread,
    io::{self, Read, Write, Stdout},
    os::unix::io::{RawFd, AsRawFd},
};

use pty::fork::{Fork, Master};
use termion::raw::{IntoRawMode, RawTerminal};
use signal_hook::iterator::Signals;
use libc;

use shell;
use fd_winsize;

// Read in page-sized chunks
const CHUNK_SIZE: usize = 1024 * 4;

pub fn fork() {
    let fork = Fork::from_ptmx().unwrap();

    match fork.is_parent() {
        // We are the master
        Ok(mut master) => {
            let mut master_clone = master.clone();
            thread::spawn(move|| {
                write_master_forever(&mut master_clone);
            });

            master_clone = master.clone();
            thread::spawn(move|| {
                handle_signals_forever(&mut master_clone);
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

/*
Signal handling function. Loops forever handling signals. Why do it this way?
Signal handler functions have strict POSIX requirements, such as not allocating
memory or touching mutexes. To satisfy these requirements, we use the
signal_hook library to self-pipe the signals into the process and collect them
in an iterator; we then iterate over them and are allowed to do whatever we
want.
*/
fn handle_signals_forever(master: &mut Master) -> Result<(), io::Error> {
    let master_fd = master.as_raw_fd();

    // Initially reset the master PTY's window size so that it matches ours; no
    // initial window size signal will be sent.
    reset_pty_winsize(master_fd);

    // Watch for incoming SIGWINCH signals, which indicate that the main
    // process's window size changed. Reset the master PTY based on the main
    // process's PTY.
    let signals = Signals::new(&[ libc::SIGWINCH ])?;

    // Loop over "all" the signals we've gotten. This is really only ever going
    // to be SIGWINCH, since it's the only one we've registered. The `forever`
    // method blocks and loops forever, so no need to throw in fake sleep
    // statements or manual loop statements.
    for signal in signals.forever() {
      match signal as libc::c_int {
        libc::SIGWINCH => {
          reset_pty_winsize(master_fd);
          ()
        },
        _ => unreachable!(),
      }
    }

    unreachable!();
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

fn reset_pty_winsize(fd: RawFd) {
    // If we successfully read a window size, set it on the PTY. If not,
    // whatever; the caller obv didn't care about the size
    fd_winsize::get(io::stdout().as_raw_fd()).and_then(|size| {
        fd_winsize::set(fd, size.rows, size.cols);
        Some(())
    });
}

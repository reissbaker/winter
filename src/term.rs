use std::{
    thread,
    io::{self, Read, Write, Stdout},
};

use pty::fork::{Fork, Master};
use termion::raw::{IntoRawMode, RawTerminal};

use shell;
use pty_win;

const CHUNK_SIZE: usize = 1024;

pub fn fork() {
    let size = pty_win::get_size();
    let fork = Fork::from_ptmx().unwrap();

    match fork.is_parent() {
        // We are the master
        Ok(mut master) => {
            let mut stdout = io::stdout().into_raw_mode().unwrap();

            let mut master_clone = master.clone();
            thread::spawn(move|| {
                write_master_forever(&mut master_clone);
            });


            match read_master_forever(&mut master, &mut stdout) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error: {:?}", e);
                },
            }
        },

        // We are the slave; exec a shell
        Err(_) => {
          match size {
            // If we previously read a window size, re-set it here.
            Some(size) => {
              pty_win::set_size(size.rows, size.cols);
            },

            // If not, whatever; the caller obv didn't care about the size
            None => (),
          }

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

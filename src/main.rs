extern crate atty;
extern crate libc;
extern crate pty;
extern crate termion;
extern crate daemonize;
extern crate signal_hook;
extern crate page_size;

mod shell;
mod term;
mod fd_winsize;

fn main() {
    term::fork();
}

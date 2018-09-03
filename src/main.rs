extern crate atty;
extern crate libc;
extern crate pty;
extern crate termion;
extern crate daemonize;

mod shell;
mod term;
mod fd_winsize;

fn main() {
    term::fork();
}

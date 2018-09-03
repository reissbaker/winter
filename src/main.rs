extern crate atty;
extern crate libc;
extern crate pty;
extern crate termion;
extern crate daemonize;

mod shell;
mod term;
mod pty_win;

fn main() {
    term::fork();
}

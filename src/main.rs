extern crate libc;
extern crate pty;
extern crate termion;
extern crate daemonize;

mod shell;
mod term;

fn main() {
    term::fork();
}

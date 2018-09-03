use std::os::unix::io::RawFd;
use libc::{
  self,
  ioctl,
  TIOCGWINSZ,
  TIOCSWINSZ,
  c_ushort,
};

// Window size struct
pub struct WindowSize {
  pub rows: u16,
  pub cols: u16,
}

// Internal struct for getting and setting window sizes from libc
#[repr(C)]
#[derive(Debug)]
struct UnixSize {
  rows: c_ushort,
  cols: c_ushort,

  // x and y are unused according to the spec, but must be present
  x: c_ushort,
  y: c_ushort,
}

// Get window size of PTY the given fd is connected to
pub fn get(fd: RawFd) -> Option<WindowSize> {
  if unsafe { libc::isatty(fd) } == 0 {
    return None;
  }

  let us = UnixSize {
    rows: 0,
    cols: 0,
    x: 0,
    y: 0,
  };

  if unsafe { ioctl(fd, TIOCGWINSZ, &us) } == 0 {
    return Some(WindowSize {
      rows: us.rows,
      cols: us.cols,
    });
  }

  None
}

// Set window size of PTY the given fd is connected to
pub fn set(fd: RawFd, rows: u16, cols: u16) -> Result<(), i32> {
  let us = UnixSize {
    rows: rows,
    cols: cols,
    x: 0,
    y: 0,
  };

  let retval = unsafe { ioctl(fd, TIOCSWINSZ, &us) };
  if retval == 0 {
    return Ok(());
  }

  Err(retval)
}

//! PTY (Pseudo-Terminal) management
//!
//! Handles creation and management of pseudo-terminals for shell processes.
//! Supports Unix (Linux, macOS) and Windows (via ConPTY).

use crate::{CoreError, Result};

/// Terminal size in rows and columns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtySize {
    pub rows: u16,
    pub cols: u16,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

impl Default for PtySize {
    fn default() -> Self {
        Self {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

// ============================================================================
// Unix implementation (Linux, macOS)
// ============================================================================

#[cfg(unix)]
mod unix {
    use super::*;
    use nix::unistd::{dup2, execvp, fork, setsid, ForkResult, Pid};
    use std::ffi::CString;
    use std::os::unix::io::RawFd;

    /// Winsize structure for ioctl
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct Winsize {
        pub ws_row: u16,
        pub ws_col: u16,
        pub ws_xpixel: u16,
        pub ws_ypixel: u16,
    }

    impl From<PtySize> for Winsize {
        fn from(size: PtySize) -> Self {
            Winsize {
                ws_row: size.rows,
                ws_col: size.cols,
                ws_xpixel: size.pixel_width,
                ws_ypixel: size.pixel_height,
            }
        }
    }

    /// A pseudo-terminal for running shell processes (Unix)
    pub struct Pty {
        /// Master file descriptor for reading/writing
        master_fd: RawFd,
        /// Child process ID
        child_pid: Pid,
        /// Current size
        size: PtySize,
    }

    impl Pty {
        /// Create a new PTY and spawn a shell process
        pub fn spawn(
            shell: Option<&str>,
            size: PtySize,
            working_dir: Option<&std::path::Path>,
            term: Option<&str>,
        ) -> Result<Self> {
            let shell = shell
                .map(String::from)
                .or_else(|| std::env::var("SHELL").ok())
                .unwrap_or_else(|| "/bin/bash".to_string());

            let term_value = term.unwrap_or("xterm-256color");

            // Open PTY master/slave pair
            let mut master_fd: libc::c_int = 0;
            let mut slave_fd: libc::c_int = 0;

            let ret = unsafe {
                libc::openpty(
                    &mut master_fd,
                    &mut slave_fd,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            };

            if ret != 0 {
                return Err(CoreError::Pty("Failed to open PTY".to_string()));
            }

            // Set initial size
            let winsize = Winsize::from(size);
            unsafe {
                libc::ioctl(slave_fd, libc::TIOCSWINSZ, &winsize);
            }

            // Fork the process
            match unsafe { fork() } {
                Ok(ForkResult::Parent { child }) => {
                    // Parent process - close slave, keep master
                    unsafe { libc::close(slave_fd) };

                    Ok(Self {
                        master_fd,
                        child_pid: child,
                        size,
                    })
                }
                Ok(ForkResult::Child) => {
                    // Child process - close master
                    unsafe { libc::close(master_fd) };

                    // Create new session
                    setsid().map_err(|e| CoreError::Pty(format!("setsid failed: {}", e)))?;

                    // Set slave as controlling terminal
                    unsafe {
                        libc::ioctl(slave_fd, libc::TIOCSCTTY, 0);
                    }

                    // Set slave as stdin/stdout/stderr
                    dup2(slave_fd, 0).map_err(|e| CoreError::Pty(format!("dup2 stdin: {}", e)))?;
                    dup2(slave_fd, 1).map_err(|e| CoreError::Pty(format!("dup2 stdout: {}", e)))?;
                    dup2(slave_fd, 2).map_err(|e| CoreError::Pty(format!("dup2 stderr: {}", e)))?;

                    if slave_fd > 2 {
                        unsafe { libc::close(slave_fd) };
                    }

                    // Change to working directory if specified
                    if let Some(dir) = working_dir {
                        if let Ok(dir_cstr) = CString::new(dir.to_string_lossy().as_bytes()) {
                            unsafe { libc::chdir(dir_cstr.as_ptr()) };
                        }
                    }

                    // Set TERM environment variable
                    if let Ok(term_cstr) = CString::new(term_value) {
                        let name_cstr = CString::new("TERM").unwrap();
                        unsafe { libc::setenv(name_cstr.as_ptr(), term_cstr.as_ptr(), 1) };
                    }

                    // Execute shell
                    let shell_cstr = CString::new(shell.as_str())
                        .map_err(|e| CoreError::Pty(format!("Invalid shell path: {}", e)))?;

                    // Use execvp via nix
                    let args: Vec<CString> = vec![shell_cstr.clone()];
                    let args_refs: Vec<&std::ffi::CStr> =
                        args.iter().map(|s| s.as_c_str()).collect();

                    execvp(&shell_cstr, &args_refs)
                        .map_err(|e| CoreError::ProcessSpawn(format!("execvp failed: {}", e)))?;

                    unreachable!()
                }
                Err(e) => {
                    unsafe {
                        libc::close(master_fd);
                        libc::close(slave_fd);
                    }
                    Err(CoreError::Pty(format!("Fork failed: {}", e)))
                }
            }
        }

        /// Get the master file descriptor for async I/O
        pub fn master_fd(&self) -> RawFd {
            self.master_fd
        }

        /// Resize the PTY
        pub fn resize(&mut self, size: PtySize) -> Result<()> {
            let winsize = Winsize::from(size);

            let ret = unsafe { libc::ioctl(self.master_fd, libc::TIOCSWINSZ, &winsize) };

            if ret < 0 {
                return Err(CoreError::Pty("Failed to resize PTY".to_string()));
            }

            self.size = size;
            Ok(())
        }

        /// Write data to the PTY (send to shell)
        pub fn write(&self, data: &[u8]) -> Result<usize> {
            let ret = unsafe {
                libc::write(
                    self.master_fd,
                    data.as_ptr() as *const libc::c_void,
                    data.len(),
                )
            };

            if ret < 0 {
                Err(CoreError::Pty("Write failed".to_string()))
            } else {
                Ok(ret as usize)
            }
        }

        /// Read data from the PTY (output from shell)
        pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
            let ret = unsafe {
                libc::read(
                    self.master_fd,
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                )
            };

            if ret < 0 {
                Err(CoreError::Pty("Read failed".to_string()))
            } else {
                Ok(ret as usize)
            }
        }

        /// Get current size
        pub fn size(&self) -> PtySize {
            self.size
        }

        /// Get child process ID
        pub fn pid(&self) -> Pid {
            self.child_pid
        }

        /// Check if child process is still running
        pub fn is_alive(&self) -> bool {
            matches!(
                nix::sys::wait::waitpid(self.child_pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG),),
                Ok(nix::sys::wait::WaitStatus::StillAlive)
            )
        }

        /// Get the foreground process group ID
        pub fn foreground_pid(&self) -> Option<Pid> {
            let pgrp = unsafe { libc::tcgetpgrp(self.master_fd) };
            if pgrp > 0 {
                Some(Pid::from_raw(pgrp))
            } else {
                None
            }
        }
    }

    impl Drop for Pty {
        fn drop(&mut self) {
            // Send SIGHUP to child process
            let _ = nix::sys::signal::kill(self.child_pid, nix::sys::signal::Signal::SIGHUP);
            // Close master fd
            unsafe { libc::close(self.master_fd) };
        }
    }
}

// ============================================================================
// Windows implementation (ConPTY)
// ============================================================================

#[cfg(windows)]
mod windows {
    use super::*;

    /// Process ID type for Windows
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Pid(u32);

    impl Pid {
        pub fn from_raw(pid: i32) -> Self {
            Pid(pid as u32)
        }

        pub fn as_raw(&self) -> i32 {
            self.0 as i32
        }
    }

    /// A pseudo-terminal for running shell processes (Windows)
    /// TODO: Implement ConPTY support
    pub struct Pty {
        size: PtySize,
        _pid: Pid,
    }

    impl Pty {
        /// Create a new PTY and spawn a shell process
        /// TODO: Implement Windows ConPTY
        pub fn spawn(
            _shell: Option<&str>,
            size: PtySize,
            _working_dir: Option<&std::path::Path>,
            _term: Option<&str>,
        ) -> Result<Self> {
            Err(CoreError::Pty(
                "Windows ConPTY support not yet implemented. Coming soon!".to_string(),
            ))
        }

        /// Get handle for async I/O (placeholder)
        pub fn master_fd(&self) -> i32 {
            -1
        }

        /// Resize the PTY
        pub fn resize(&mut self, size: PtySize) -> Result<()> {
            self.size = size;
            Ok(())
        }

        /// Write data to the PTY
        pub fn write(&self, _data: &[u8]) -> Result<usize> {
            Err(CoreError::Pty("Windows PTY not implemented".to_string()))
        }

        /// Read data from the PTY
        pub fn read(&self, _buf: &mut [u8]) -> Result<usize> {
            Err(CoreError::Pty("Windows PTY not implemented".to_string()))
        }

        /// Get current size
        pub fn size(&self) -> PtySize {
            self.size
        }

        /// Get child process ID
        pub fn pid(&self) -> Pid {
            self._pid
        }

        /// Check if child process is still running
        pub fn is_alive(&self) -> bool {
            false
        }

        /// Get the foreground process group ID
        pub fn foreground_pid(&self) -> Option<Pid> {
            None
        }
    }
}

// ============================================================================
// Re-exports
// ============================================================================

#[cfg(unix)]
pub use unix::Pty;

#[cfg(unix)]
pub use unix::Winsize;

#[cfg(windows)]
pub use windows::Pty;

#[cfg(windows)]
pub use windows::Pid;

// On Unix, re-export Pid from nix
#[cfg(unix)]
pub use nix::unistd::Pid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_size_default() {
        let size = PtySize::default();
        assert_eq!(size.rows, 24);
        assert_eq!(size.cols, 80);
    }

    #[cfg(unix)]
    #[test]
    fn test_winsize_conversion() {
        let size = PtySize {
            rows: 40,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        };
        let winsize = Winsize::from(size);
        assert_eq!(winsize.ws_row, 40);
        assert_eq!(winsize.ws_col, 120);
    }
}

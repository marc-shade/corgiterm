//! Cross-Platform PTY (Pseudo-Terminal) management
//!
//! Provides a unified interface for PTY operations across platforms:
//! - Linux: Uses native PTY via portable-pty
//! - macOS: Uses native PTY via portable-pty
//! - Windows: Uses ConPTY via portable-pty
//!
//! This abstraction enables CorgiTerm to run on all major platforms.

use crate::{CoreError, Result};
use portable_pty::{
    native_pty_system, Child, CommandBuilder, MasterPty, PtySize as PortablePtySize,
};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

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

impl From<PtySize> for PortablePtySize {
    fn from(size: PtySize) -> Self {
        PortablePtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: size.pixel_width,
            pixel_height: size.pixel_height,
        }
    }
}

/// A cross-platform pseudo-terminal for running shell processes
pub struct Pty {
    /// Master PTY handle (wrapped for thread safety)
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    /// Reader for the PTY (wrapped for thread safety)
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    /// Writer for the PTY (wrapped for thread safety)
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    /// Child process handle
    child: Arc<Mutex<Box<dyn Child + Send + Sync>>>,
    /// Current size
    size: PtySize,
    /// Process ID (platform-specific)
    #[cfg(unix)]
    pid: Option<nix::unistd::Pid>,
    #[cfg(not(unix))]
    pid: Option<u32>,
}

impl Pty {
    /// Create a new PTY and spawn a shell process
    ///
    /// Works on Linux, macOS, and Windows.
    /// - `shell`: Optional shell to use. Defaults to $SHELL on Unix, cmd.exe/pwsh on Windows.
    /// - `size`: Initial terminal size.
    /// - `working_dir`: Optional starting directory.
    /// - `term`: Optional TERM environment variable (Unix only).
    pub fn spawn(
        shell: Option<&str>,
        size: PtySize,
        working_dir: Option<&std::path::Path>,
        term: Option<&str>,
    ) -> Result<Self> {
        // Get native PTY system (works on all platforms)
        let pty_system = native_pty_system();

        // Open PTY pair with initial size
        let pair = pty_system
            .openpty(size.into())
            .map_err(|e| CoreError::Pty(format!("Failed to open PTY: {}", e)))?;

        // Determine shell to use
        let shell_path = shell
            .map(String::from)
            .or_else(|| std::env::var("SHELL").ok())
            .unwrap_or_else(|| {
                #[cfg(unix)]
                {
                    "/bin/bash".to_string()
                }
                #[cfg(windows)]
                {
                    // Prefer PowerShell if available, fall back to cmd
                    std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
                }
            });

        // Build command
        let mut cmd = CommandBuilder::new(&shell_path);

        // Set working directory if specified
        if let Some(dir) = working_dir {
            cmd.cwd(dir);
        }

        // Set TERM environment variable (Unix)
        #[cfg(unix)]
        {
            let term_value = term.unwrap_or("xterm-256color");
            cmd.env("TERM", term_value);
        }

        // Spawn child process
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| CoreError::ProcessSpawn(format!("Failed to spawn shell: {}", e)))?;

        // Get process ID
        #[cfg(unix)]
        let pid = child
            .process_id()
            .map(|id| nix::unistd::Pid::from_raw(id as i32));
        #[cfg(not(unix))]
        let pid = child.process_id();

        // Get reader/writer handles
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| CoreError::Pty(format!("Failed to clone PTY reader: {}", e)))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| CoreError::Pty(format!("Failed to take PTY writer: {}", e)))?;

        Ok(Self {
            master: Arc::new(Mutex::new(pair.master)),
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            child: Arc::new(Mutex::new(child)),
            size,
            pid,
        })
    }

    /// Resize the PTY
    pub fn resize(&mut self, size: PtySize) -> Result<()> {
        let master = self
            .master
            .lock()
            .map_err(|_| CoreError::Pty("Lock poisoned".to_string()))?;
        master
            .resize(size.into())
            .map_err(|e| CoreError::Pty(format!("Failed to resize PTY: {}", e)))?;
        self.size = size;
        Ok(())
    }

    /// Write data to the PTY (send to shell)
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| CoreError::Pty("Lock poisoned".to_string()))?;
        writer
            .write(data)
            .map_err(|e| CoreError::Pty(format!("Write failed: {}", e)))
    }

    /// Read data from the PTY (output from shell)
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let mut reader = self
            .reader
            .lock()
            .map_err(|_| CoreError::Pty("Lock poisoned".to_string()))?;
        reader
            .read(buf)
            .map_err(|e| CoreError::Pty(format!("Read failed: {}", e)))
    }

    /// Get current size
    pub fn size(&self) -> PtySize {
        self.size
    }

    /// Get child process ID (Unix)
    #[cfg(unix)]
    pub fn pid(&self) -> nix::unistd::Pid {
        self.pid.unwrap_or(nix::unistd::Pid::from_raw(0))
    }

    /// Get child process ID (Windows)
    #[cfg(not(unix))]
    pub fn pid(&self) -> u32 {
        self.pid.unwrap_or(0)
    }

    /// Check if child process is still running
    pub fn is_alive(&self) -> bool {
        if let Ok(mut child) = self.child.lock() {
            // try_wait returns None if still running
            match child.try_wait() {
                Ok(None) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Get the master file descriptor for async I/O (Unix only)
    ///
    /// Note: portable-pty abstracts the underlying fd, so direct access
    /// is not available. Returns -1 to indicate async handling is required.
    /// Users should use the read/write methods instead.
    #[cfg(unix)]
    #[allow(dead_code)]
    pub fn master_fd(&self) -> std::os::unix::io::RawFd {
        // portable-pty doesn't expose the underlying fd directly
        // This is intentional - it maintains cross-platform compatibility
        -1
    }

    /// Get the foreground process group ID (Unix only)
    #[cfg(unix)]
    pub fn foreground_pid(&self) -> Option<nix::unistd::Pid> {
        // This requires access to the raw fd which portable-pty abstracts
        // For cross-platform compatibility, this might not be available
        None
    }

    /// Kill the child process
    pub fn kill(&self) -> Result<()> {
        let mut child = self
            .child
            .lock()
            .map_err(|_| CoreError::Pty("Lock poisoned".to_string()))?;
        child
            .kill()
            .map_err(|e| CoreError::Pty(format!("Failed to kill process: {}", e)))
    }

    /// Wait for the child process to exit
    pub fn wait(&self) -> Result<portable_pty::ExitStatus> {
        let mut child = self
            .child
            .lock()
            .map_err(|_| CoreError::Pty("Lock poisoned".to_string()))?;
        child
            .wait()
            .map_err(|e| CoreError::Pty(format!("Failed to wait for process: {}", e)))
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        // Kill the child process on drop
        let _ = self.kill();
    }
}

// Ensure Pty is Send + Sync for async usage
unsafe impl Send for Pty {}
unsafe impl Sync for Pty {}

/// Winsize structure for legacy compatibility (Unix only)
#[cfg(unix)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Winsize {
    pub ws_row: u16,
    pub ws_col: u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
}

#[cfg(unix)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_size_default() {
        let size = PtySize::default();
        assert_eq!(size.rows, 24);
        assert_eq!(size.cols, 80);
    }

    #[test]
    fn test_pty_size_conversion() {
        let size = PtySize {
            rows: 40,
            cols: 120,
            pixel_width: 800,
            pixel_height: 600,
        };
        let portable_size: PortablePtySize = size.into();
        assert_eq!(portable_size.rows, 40);
        assert_eq!(portable_size.cols, 120);
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

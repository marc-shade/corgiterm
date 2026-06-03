#![cfg(unix)]

use std::io::Read;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use corgiterm_core::{Pty, PtySize};

#[test]
fn spawned_shell_writes_output_through_pty() {
    let pty = Pty::spawn(
        Some("/bin/sh"),
        PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        },
        None,
        Some("xterm-256color"),
    )
    .expect("shell should spawn inside a PTY");

    let reader = pty.reader_clone();
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let reader_thread = std::thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            let read = {
                let mut guard = reader
                    .lock()
                    .expect("PTY reader lock should not be poisoned");
                guard.read(&mut buf)
            };
            match read {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    pty.write(b"printf 'CORGI_PTY_OK\\n'\nexit\n")
        .expect("test command should be written to PTY");

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut output = Vec::new();
    while Instant::now() < deadline {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(chunk) => {
                output.extend_from_slice(&chunk);
                if String::from_utf8_lossy(&output).contains("CORGI_PTY_OK") {
                    let _ = reader_thread.join();
                    return;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let _ = pty.kill();
    let _ = reader_thread.join();
    panic!(
        "PTY output did not contain expected marker. Output was: {}",
        String::from_utf8_lossy(&output)
    );
}

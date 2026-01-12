use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty, Child};
use tokio::sync::broadcast;

pub struct PtyManager {
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    reader: Mutex<Option<Box<dyn Read + Send>>>,
    child: Mutex<Option<Box<dyn Child + Send>>>,
}

impl PtyManager {
        #[must_use]
        pub fn new() -> Self {
            let pty_system = NativePtySystem::default();
            let pair = pty_system
                .openpty(PtySize {
                    rows: 24,
                    cols: 80,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .unwrap();
    
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
            let mut cmd = CommandBuilder::new(shell);
            cmd.env("TERM", "xterm-256color");
            cmd.env("COLORTERM", "truecolor");
            cmd.env("LANG", "C.UTF-8");
    
            let child = pair.slave.spawn_command(cmd).unwrap();
            
            #[cfg(unix)]
            {
                if let Some(pid) = child.process_id() {
                    let pid = pid.cast_signed();
                    // Watcher thread: kill child group if parent (bterminal) dies
                    thread::spawn(move || {
                        let parent_pid = unsafe { libc::getppid() };
                        loop {
                            thread::sleep(Duration::from_millis(500));
                            let current_parent = unsafe { libc::getppid() };
                            if current_parent != parent_pid {
                                // Parent gone, kill child group
                                use nix::sys::signal::{kill, Signal};
                                use nix::unistd::Pid;
                                let _ = kill(Pid::from_raw(-pid), Signal::SIGKILL);
                                break;
                            }
                        }
                    });
                }
            }
    
            let writer = pair.master.take_writer().unwrap();
            let reader = pair.master.try_clone_reader().unwrap();
            let master = pair.master;
    
            Self {
                master: Arc::new(Mutex::new(master)),
                writer: Arc::new(Mutex::new(writer)),
                reader: Mutex::new(Some(reader)),
                child: Mutex::new(Some(child)),
            }
        }

    /// Shutdown the PTY and kill the associated process group.
    pub fn shutdown(&self) {
        let mut child_lock = self.child.lock().unwrap();
        if let Some(mut child) = child_lock.take() {
            #[cfg(unix)]
            if let Some(pid) = child.process_id() {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                // Kill the entire process group. 
                // portable_pty calls setsid() so the child is a PGID leader.
                let _ = kill(Pid::from_raw(-pid.cast_signed()), Signal::SIGKILL);
            }
            // Reap the child process to avoid zombies.
            let _ = child.wait();
        }
    }

    pub fn start_reader(&self, tx: broadcast::Sender<Vec<u8>>) {
        let mut reader_opt = self.reader.lock().unwrap();
        if let Some(mut reader) = reader_opt.take() {
            thread::spawn(move || {
                let mut buf = [0u8; 1024];
                while let Ok(n) = reader.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    if let Err(e) = tx.send(buf[..n].to_vec()) {
                        println!("Broadcast error (expected if no listeners): {e}");
                    }
                }
                println!("PTY Reader thread exiting.");
                // Send an empty vector to signal termination to subscribers
                let _ = tx.send(Vec::new());
            });
        }
    }

    /// # Errors
    /// Returns error if PTY writer fails
    pub fn write(&self, data: &[u8]) -> anyhow::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(data)?;
        writer.flush()?;
        drop(writer);
        Ok(())
    }

    pub fn resize(&self, rows: u16, cols: u16) -> anyhow::Result<()> {
        let master = self.master.lock().unwrap();
        master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        drop(master);
        Ok(())
    }
}

impl Drop for PtyManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;
    use std::time::Duration;

    #[tokio::test]
    async fn test_process_group_cleanup() {
        let pty = PtyManager::new();
        let (tx, mut rx) = broadcast::channel(100);
        pty.start_reader(tx);

        // Run a long-running process in the background
        // Disable job control (monitor mode) so background jobs stay in the shell's PGID
        pty.write(b"set +m\n").unwrap();
        pty.write(b"sleep 100 &\n").unwrap();
        pty.write(b"SLEEP_PID=$!\n").unwrap();
        // Use a very specific marker that won't be easily confused with the echoed command
        pty.write(b"echo \"###PID:$SLEEP_PID:###\"\n").unwrap();

        let mut accumulated_output = String::new();
        let timeout = tokio::time::sleep(Duration::from_secs(5));
        tokio::pin!(timeout);

        let sleep_pid = 'outer: loop {
            tokio::select! {
                msg = rx.recv() => {
                    if let Ok(data) = msg {
                        accumulated_output.push_str(&String::from_utf8_lossy(&data));
                        
                        let mut search_pos = 0;
                        while let Some(start_pos) = accumulated_output[search_pos..].find("###PID:") {
                            let actual_start = search_pos + start_pos;
                            let tail = &accumulated_output[actual_start + 7..];
                            if let Some(end_pos) = tail.find(":###") {
                                let pid_str = &tail[..end_pos];
                                if let Ok(pid) = pid_str.parse::<i32>() {
                                    break 'outer pid;
                                }
                                search_pos = actual_start + 7 + end_pos + 4;
                            } else {
                                break;
                            }
                        }
                    }
                }
                _ = &mut timeout => panic!("Failed to get sleep PID. Accumulated: {}", accumulated_output),
            }
        };

        assert!(sleep_pid > 0);

        // Check that sleep is running
        assert!(nix::sys::signal::kill(nix::unistd::Pid::from_raw(sleep_pid), None).is_ok());

        // Now shutdown the PTY (this should kill the whole group)
        pty.shutdown();

        // Check that sleep is killed
        thread::sleep(Duration::from_millis(200));
        let res = nix::sys::signal::kill(nix::unistd::Pid::from_raw(sleep_pid), None);
        assert!(res.is_err(), "Sleep process should have been killed");
    }

    #[tokio::test]
    async fn test_pty_termination_signal() {
        let pty = PtyManager::new();
        let (tx, mut rx) = broadcast::channel(10);
        
        // Start reading
        pty.start_reader(tx);
        
        // Send 'exit\n' to the PTY
        pty.write(b"exit\n").unwrap();
        
        let mut found_termination = false;
        let timeout = tokio::time::sleep(Duration::from_secs(5));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                msg = rx.recv() => {
                    match msg {
                        Ok(data) => {
                            if data.is_empty() {
                                found_termination = true;
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                () = &mut timeout => {
                    break;
                }
            }
        }

        assert!(found_termination, "Should have received an empty vector termination signal");
    }

    #[test]
    fn test_pty_write_and_resize() {
        let pty = PtyManager::default();
        
        // Test write
        assert!(pty.write(b"ls\n").is_ok());
        
        // Test resize
        assert!(pty.resize(24, 80).is_ok());
        assert!(pty.resize(40, 120).is_ok());
    }

    #[tokio::test]
    async fn test_pty_env_vars() {
        let pty = PtyManager::new();
        let (tx, mut rx) = broadcast::channel(100);
        pty.start_reader(tx);

        // We use a small delay to let the shell initialize
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Write command to print env vars
        pty.write(b"echo $TERM $COLORTERM $LANG\n").unwrap();

        let mut output = String::new();
        let timeout = tokio::time::sleep(Duration::from_secs(2));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                msg = rx.recv() => {
                    if let Ok(data) = msg {
                        output.push_str(&String::from_utf8_lossy(&data));
                        if output.contains("xterm-256color") && 
                           output.contains("truecolor") && 
                           output.contains(".UTF-8") {
                            return; // Success
                        }
                    } else {
                        break;
                    }
                }
                _ = &mut timeout => {
                    panic!("Timeout waiting for env vars in output. Got: {}", output);
                }
            }
        }
    }
}
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::broadcast;

pub struct PtyInstance {
    pub master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub tx: broadcast::Sender<Vec<u8>>,
}

impl PtyInstance {
    pub fn new(shell: &str, size: PtySize) -> anyhow::Result<Self> {
        println!("Creating new PTY session with shell: {}", shell);
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(size)?;

        let cmd = CommandBuilder::new(shell);
        let mut child = pair.slave.spawn_command(cmd)?;

        let master = pair.master;
        let writer = master.take_writer()?;
        let (tx, _rx) = broadcast::channel(1024);

            let master_arc: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>> = Arc::new(Mutex::new(master));
            let writer_arc = Arc::new(Mutex::new(writer));
            
            let master_clone = Arc::clone(&master_arc);
            let tx_clone = tx.clone();
            let shell_string = shell.to_string();

            thread::spawn(move || {
                println!("PTY reader thread started for shell: {}", shell_string);
                let mut reader = master_clone.lock().unwrap().try_clone_reader().unwrap();
                let mut buffer = [0u8; 1024];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => {
                            println!("PTY reader EOF");
                            break;
                        }
                        Ok(n) => {
                            println!("PTY reader received {} bytes.", n);
                            if tx_clone.receiver_count() > 0 || true { // Always keep buffer updated via session
                                 let _ = tx_clone.send(buffer[..n].to_vec());
                            }
                        }
                        Err(e) => {
                            println!("PTY reader error: {}", e);
                            break;
                        }
                    }
                }
                let _ = child.wait();
                println!("PTY process exited");
            });

            Ok(Self {
                master: master_arc,
                writer: writer_arc,
                tx,
            })
        }

        pub fn write(&self, data: &[u8]) -> anyhow::Result<()> {
            println!("PTY writer received {} bytes.", data.len());
            let mut writer = self.writer.lock().unwrap();
            writer.write_all(data)?;
            writer.flush()?;
            Ok(())
        }

        pub fn resize(&self, rows: u16, cols: u16) -> anyhow::Result<()> {
            println!("PTY resize to {}x{}", cols, rows);
            self.master.lock().unwrap().resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;
            Ok(())
        }
    }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_creation_and_write() {
        let size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };
        // Use /bin/sh for broader compatibility in tests
        let pty = PtyInstance::new("/bin/sh", size);
        assert!(pty.is_ok());
        
        let pty = pty.unwrap();
        // Test write (should not panic)
        let res = pty.write(b"ls\n");
        assert!(res.is_ok());
    }
}

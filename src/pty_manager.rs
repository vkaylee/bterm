use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty};
use tokio::sync::broadcast;

pub struct PtyManager {
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    reader: Mutex<Option<Box<dyn Read + Send>>>,
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
        let cmd = CommandBuilder::new(shell);
        let mut child = pair.slave.spawn_command(cmd).unwrap();

        thread::spawn(move || {
            let _ = child.wait();
        });

        let writer = pair.master.take_writer().unwrap();
        let reader = pair.master.try_clone_reader().unwrap();
        let master = pair.master;

        Self {
            master: Arc::new(Mutex::new(master)),
            writer: Arc::new(Mutex::new(writer)),
            reader: Mutex::new(Some(reader)),
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
            });
        }
    }

    /// # Errors
    /// Returns error if PTY writer fails
    pub fn write(&self, data: &[u8]) -> anyhow::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(data)?;
        writer.flush()?;
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
        Ok(())
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}
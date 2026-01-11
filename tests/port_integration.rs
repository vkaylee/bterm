use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use std::time::Duration;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_port_env_variable() {
    // Tìm một port trống để test
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Giải phóng port

    let mut child = Command::new("cargo")
        .arg("run")
        .env("PORT", port.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn bterminal");

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    let mut found = false;
    let expected_msg = format!("running on http://localhost:{port}");

    // Chờ tối đa 30s để compile và chạy
    let timeout = tokio::time::sleep(Duration::from_secs(30));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            line = reader.next_line() => {
                if let Ok(Some(l)) = line {
                    if l.contains(&expected_msg) {
                        found = true;
                        break;
                    }
                } else {
                    break;
                }
            }
            _ = &mut timeout => break,
        }
    }

    let _ = child.kill().await;
    assert!(found, "Should have started on port {} from environment variable", port);
}

#[tokio::test]
async fn test_port_collision_fallback() {
    // 1. Chiếm port 3000
    let _occupier = TcpListener::bind("0.0.0.0:3000").await
        .expect("Could not bind to 3000 for test");

    // 2. Chạy app (mặc định sẽ thử 3000 rồi fallback)
    let mut child = Command::new("cargo")
        .arg("run")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn bterminal");

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    let mut detected_port = 0;
    
    let timeout = tokio::time::sleep(Duration::from_secs(30));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            line = reader.next_line() => {
                if let Ok(Some(l)) = line {
                    if l.contains("running on http://localhost:") {
                        let parts: Vec<&str> = l.split(':').collect();
                        if let Some(port_str) = parts.last() {
                            detected_port = port_str.parse::<u16>().unwrap_or(0);
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
            _ = &mut timeout => break,
        }
    }

    let _ = child.kill().await;
    assert!(detected_port != 0, "Should have detected a port");
    assert!(detected_port != 3000, "Should NOT have used port 3000 as it was occupied");
}

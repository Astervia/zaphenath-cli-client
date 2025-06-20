use std::{fs, path::Path, process::Command, thread, time::Duration};

#[test]
fn test_daemon_detached_and_stop() {
    let _ = dotenvy::from_filename(".env.test");

    // Clean up any old state
    let _ = fs::remove_file(".zaphenathd.pid");
    let _ = fs::remove_file(".zaphenathd.log");

    // Start the daemon in detached mode
    let mut daemon = Command::new("cargo");
    daemon.args([
        "run",
        "--quiet",
        "--",
        "daemon",
        "run",
        "--interval",
        "1",
        "--detached",
        "--nonce",
        "700",
        "--gas-buffer",
        "1.2",
    ]);
    let child = daemon.spawn().expect("Failed to start detached daemon");
    drop(child); // ✅ Explicitly drop the child to silence clippy warning

    // Wait a bit for the daemon to start
    thread::sleep(Duration::from_secs(3));

    // ✅ Check if PID file exists
    assert!(
        Path::new(".zaphenathd.pid").exists(),
        "❌ PID file not created"
    );

    // ✅ Check if log file exists and has expected content
    let log_contents = fs::read_to_string(".zaphenathd.log").expect("❌ Log file not found");
    assert!(
        log_contents.contains("Daemon started"),
        "❌ Daemon startup log not found"
    );

    // 🛑 Send stop signal via CLI
    let stop_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "daemon", "stop"])
        .status()
        .expect("Failed to run daemon stop");

    assert!(stop_status.success(), "❌ Stop command failed");

    // Wait briefly for cleanup
    thread::sleep(Duration::from_secs(1));

    // ✅ PID file should be gone
    assert!(
        !Path::new(".zaphenathd.pid").exists(),
        "❌ PID file still exists after stop"
    );
}

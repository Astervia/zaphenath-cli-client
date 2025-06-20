#[cfg(unix)]
use nix::sys::signal::{Signal, kill};
#[cfg(unix)]
use nix::unistd::Pid;
use std::{fs, path::Path};

pub fn stop_daemon() {
    let pid_path = ".zaphenathd.pid";

    if !Path::new(pid_path).exists() {
        eprintln!("⚠️ PID file not found. Is the daemon running?");
        return;
    }

    let pid_str = match fs::read_to_string(pid_path) {
        Ok(p) => p.trim().to_string(),
        Err(e) => {
            eprintln!("❌ Failed to read PID file: {e}");
            return;
        }
    };

    let pid: u32 = match pid_str.parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("❌ Invalid PID in file: {pid_str}");
            return;
        }
    };

    #[cfg(unix)]
    {
        match kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
            Ok(_) => {
                println!("🛑 Sent SIGTERM to process {pid}");
                let _ = fs::remove_file(pid_path);
            }
            Err(e) => {
                eprintln!("❌ Failed to kill process {pid}: {e}");
            }
        }
    }

    #[cfg(windows)]
    {
        let output = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match output {
            Ok(status) if status.success() => {
                println!("🛑 Terminated process {pid}");
                let _ = fs::remove_file(pid_path);
            }
            Ok(_) | Err(_) => {
                eprintln!("❌ Failed to terminate process {pid}");
            }
        }
    }
}

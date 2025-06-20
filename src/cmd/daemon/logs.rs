use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;

pub fn show_logs() {
    let log_path = ".zaphenathd.log";

    if !Path::new(log_path).exists() {
        eprintln!("⚠️ Log file not found.");
        return;
    }

    // Try to use `less` if available (Unix only)
    #[cfg(unix)]
    {
        if which::which("less").is_ok() {
            let _ = Command::new("less")
                .arg(log_path)
                .status()
                .unwrap_or_else(|_| {
                    eprintln!("❌ Failed to launch pager (less)");
                    std::process::exit(1);
                });
            return;
        }
    }

    // Fallback: Print the log contents line-by-line
    match fs::File::open(log_path) {
        Ok(file) => {
            println!("📜 Showing log file: {}\n", log_path);
            for line in io::BufReader::new(file).lines().flatten() {
                println!("{line}");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to read log file: {e}");
        }
    }
}

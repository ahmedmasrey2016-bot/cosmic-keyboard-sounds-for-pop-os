use anyhow::{Context, Result};
use std::path::PathBuf;

fn service_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .context("HOME not set")?;
    let dir = home.join(".config/systemd/user");
    std::fs::create_dir_all(&dir).context("create systemd user dir")?;
    Ok(dir.join("cosmic-keyboard-sounds.service"))
}

fn binary_path() -> Result<PathBuf> {
    // Try to find ourselves; fall back to /usr/local/bin
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("/usr/local/bin/cosmic-keyboard-sounds"));
    Ok(exe)
}

pub fn install_service() -> Result<()> {
    let bin = binary_path()?;
    let path = service_path()?;

    let unit = format!(
        r#"[Unit]
Description=Mechanical keyboard sounds
Documentation=https://github.com/cosmic-keyboard-sounds
After=sound.target pipewire.service pulseaudio.service
Wants=sound.target

[Service]
Type=simple
ExecStart={bin}
Restart=on-failure
RestartSec=3
Environment=RUST_LOG=warn

[Install]
WantedBy=default.target
"#,
        bin = bin.display()
    );

    std::fs::write(&path, unit).context("write service file")?;

    // Reload daemon and enable
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "enable", "cosmic-keyboard-sounds"])
        .status();

    println!("Service file written to: {}", path.display());
    Ok(())
}

pub fn uninstall_service() -> Result<()> {
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "cosmic-keyboard-sounds"])
        .status();

    let path = service_path()?;
    if path.exists() {
        std::fs::remove_file(&path).context("remove service file")?;
        println!("Removed: {}", path.display());
    } else {
        println!("Service file not found (already removed).");
    }

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    Ok(())
}

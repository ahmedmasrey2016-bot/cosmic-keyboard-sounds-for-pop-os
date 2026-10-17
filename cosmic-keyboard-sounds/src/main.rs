mod input;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cosmic_keyboard_sounds::config::Config;
use cosmic_keyboard_sounds::{audio, install};
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Parser)]
#[command(name = "cosmic-keyboard-sounds", version = "0.2.0",
          about = "🎹 Mechanical keyboard sounds for Linux")]
struct Cli {
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Start listening and playing sounds (default)
    Run,
    /// Install as a systemd user service
    Install,
    /// Remove the systemd user service
    Uninstall,
    /// Set the sound theme
    Theme {
        /// cherry-mx-blue | cherry-mx-brown | cherry-mx-red | typewriter
        name: String,
    },
    /// Set the volume (0.0 – 2.0)
    Volume { level: f32 },
    /// List available themes
    Themes,
    /// Show current config
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(tracing::metadata::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let config_path = cosmic_keyboard_sounds::config::config_path();

    match cli.command.unwrap_or(Cmd::Run) {
        Cmd::Run => run(config_path)?,

        Cmd::Install => {
            install::install_service()?;
            println!("✅ Installed. Start now: systemctl --user start cosmic-keyboard-sounds");
        }
        Cmd::Uninstall => {
            install::uninstall_service()?;
            println!("✅ Removed.");
        }

        Cmd::Theme { name } => {
            let mut cfg = Config::load(&config_path).unwrap_or_default();
            cfg.theme = name.parse().map_err(|e: String| anyhow::anyhow!(e))?;
            cfg.save(&config_path)?;
            println!("✅ Theme → {}", cfg.theme);
        }
        Cmd::Volume { level } => {
            if !(0.0..=2.0).contains(&level) {
                anyhow::bail!("Volume must be 0.0–2.0");
            }
            let mut cfg = Config::load(&config_path).unwrap_or_default();
            cfg.volume = level;
            cfg.save(&config_path)?;
            println!("✅ Volume → {:.0}%", level * 100.0);
        }
        Cmd::Themes => {
            println!("cherry-mx-blue   🔵 Clicky");
            println!("cherry-mx-brown  🟤 Tactile");
            println!("cherry-mx-red    🔴 Silent");
            println!("typewriter       🖊️  Vintage");
        }
        Cmd::Status => {
            let cfg = Config::load(&config_path).unwrap_or_default();
            println!("Config: {:?}", config_path);
            println!("  Theme:  {}", cfg.theme);
            println!("  Volume: {:.0}%", cfg.volume * 100.0);
        }
    }

    Ok(())
}

fn run(config_path: std::path::PathBuf) -> Result<()> {
    let config = match Config::load(&config_path) {
        Ok(c) => { info!("Loaded config from {:?}", config_path); c }
        Err(_) => {
            let c = Config::default();
            let _ = c.save(&config_path);
            c
        }
    };

    if !config.enabled {
        info!("Disabled in config. Exiting.");
        return Ok(());
    }

    info!("🎹 cosmic-keyboard-sounds v0.2.0 | {} | {:.0}% vol",
          config.theme, config.volume * 100.0);

    let audio = match audio::AudioEngine::new(config.clone()) {
        Ok(a) => Arc::new(a),
        Err(e) => { error!("Audio init failed: {}", e); return Err(e); }
    };

    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || { r.store(false, std::sync::atomic::Ordering::SeqCst); })?;

    info!("Listening… (Ctrl+C to stop)");
    input::listen(audio, running, &config)?;
    Ok(())
}

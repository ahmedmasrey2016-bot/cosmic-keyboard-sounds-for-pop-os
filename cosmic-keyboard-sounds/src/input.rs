use crate::audio::AudioEngine;
use cosmic_keyboard_sounds::config::Config;
use anyhow::Result;
use evdev::{Device, EventType, InputEventKind, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

fn find_keyboards() -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = glob::glob("/dev/input/event*") else { return found };

    for path in entries.flatten() {
        if let Ok(device) = Device::open(&path) {
            let supported = device.supported_events();
            if supported.contains(EventType::KEY) {
                if let Some(keys) = device.supported_keys() {
                    if keys.contains(Key::KEY_A) && keys.contains(Key::KEY_ENTER) {
                        info!(
                            "Found keyboard: {:?} → {}",
                            device.name().unwrap_or("unknown"),
                            path.display()
                        );
                        found.push(path);
                    }
                }
            }
        }
    }
    found
}

pub fn listen(
    audio: Arc<AudioEngine>,
    running: Arc<AtomicBool>,
    config: &Config,
) -> Result<()> {
    let keyboards = find_keyboards();
    if keyboards.is_empty() {
        anyhow::bail!(
            "No keyboard devices found.\n\
             Make sure you are in the 'input' group:\n\
             \n  sudo usermod -aG input $USER\n\
             \nThen log out and back in."
        );
    }

    info!("Monitoring {} keyboard device(s)", keyboards.len());

    let mut handles = Vec::new();

    for path in keyboards {
        let audio       = Arc::clone(&audio);
        let running     = Arc::clone(&running);
        let play_press   = config.play_on_press;
        let play_release = config.play_on_release;
        let key_filter   = config.key_filter.clone();

        let handle = std::thread::Builder::new()
            .name(format!("kbd:{}", path.display()))
            .spawn(move || {
                let mut device = match Device::open(&path) {
                    Ok(d) => d,
                    Err(e) => { warn!("Cannot open {}: {}", path.display(), e); return; }
                };

                while running.load(Ordering::SeqCst) {
                    match device.fetch_events() {
                        Ok(events) => {
                            for ev in events {
                                let InputEventKind::Key(key) = ev.kind() else { continue };
                                let code = key.0;

                                if !key_filter.is_empty() && !key_filter.contains(&code) {
                                    continue;
                                }

                                match ev.value() {
                                    1 if play_press   => audio.play_press(code),
                                    0 if play_release => audio.play_release(code),
                                    _ => {}
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Read error on {}: {}", path.display(), e);
                            std::thread::sleep(std::time::Duration::from_millis(200));
                        }
                    }
                }
            })?;

        handles.push(handle);
    }

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    for h in handles { let _ = h.join(); }
    Ok(())
}

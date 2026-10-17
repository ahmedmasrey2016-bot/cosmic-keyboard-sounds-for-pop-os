use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GBox, Button, ComboBoxText, Label,
    Orientation, Scale, Switch, Separator, Align,
    glib, gio,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::process::Command;

// Import shared config from the main crate
use cosmic_keyboard_sounds::config::{Config, Theme, config_path};
use cosmic_keyboard_sounds::audio::AudioEngine;

const APP_ID: &str = "com.system76.CosmicKeyboardSoundsGui";

pub fn run() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    // Load config
    let cfg_path = config_path();
    let config = Rc::new(RefCell::new(
        Config::load(&cfg_path).unwrap_or_default()
    ));

    // ── Window ───────────────────────────────────────────────────────────────
    let window = ApplicationWindow::builder()
        .application(app)
        .title("🎹 Keyboard Sounds")
        .default_width(420)
        .default_height(480)
        .resizable(false)
        .build();

    // ── Root container ───────────────────────────────────────────────────────
    let root = GBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // ── Header ───────────────────────────────────────────────────────────────
    let header = GBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(24)
        .margin_bottom(16)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title_label = Label::builder()
        .label("🎹 Keyboard Sounds")
        .halign(Align::Start)
        .build();
    title_label.add_css_class("title-1");

    let subtitle_label = Label::builder()
        .label("Mechanical keyboard sound effects for COSMIC")
        .halign(Align::Start)
        .build();
    subtitle_label.add_css_class("dim-label");

    header.append(&title_label);
    header.append(&subtitle_label);
    root.append(&header);
    root.append(&Separator::new(Orientation::Horizontal));

    // ── Content ──────────────────────────────────────────────────────────────
    let content = GBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(24)
        .margin_end(24)
        .build();

    // ── Enable / Disable row ──────────────────────────────────────────────────
    {
        let row = make_row("Enable Keyboard Sounds", "Play sounds on every keypress");
        let sw = Switch::builder()
            .active(config.borrow().enabled)
            .valign(Align::Center)
            .build();

        let cfg = Rc::clone(&config);
        let cfg_path2 = cfg_path.clone();
        sw.connect_active_notify(move |s| {
            cfg.borrow_mut().enabled = s.is_active();
            save_config(&cfg, &cfg_path2);
            restart_daemon();
        });

        row.append(&sw);
        content.append(&row);
    }

    content.append(&Separator::new(Orientation::Horizontal));

    // ── Theme selector ────────────────────────────────────────────────────────
    {
        let row = make_row("Sound Theme", "Character of the switch sound");
        let combo = ComboBoxText::new();

        let themes = [
            ("cherry-mx-blue",  "🔵 Cherry MX Blue — Clicky"),
            ("cherry-mx-brown", "🟤 Cherry MX Brown — Tactile"),
            ("cherry-mx-red",   "🔴 Cherry MX Red — Silent"),
            ("typewriter",      "🖊️  Typewriter — Vintage"),
        ];

        let current_theme = format!("{}", config.borrow().theme)
            .split_whitespace().next().unwrap_or("cherry-mx-blue")
            .to_string();

        for (id, label) in &themes {
            combo.append(Some(id), label);
        }
        combo.set_active_id(Some(&current_theme));

        let cfg = Rc::clone(&config);
        let cfg_path2 = cfg_path.clone();
        combo.connect_changed(move |c| {
            if let Some(id) = c.active_id() {
                if let Ok(theme) = id.as_str().parse::<Theme>() {
                    cfg.borrow_mut().theme = theme;
                    save_config(&cfg, &cfg_path2);
                    restart_daemon();
                }
            }
        });

        row.append(&combo);
        content.append(&row);
    }

    // ── Volume slider ─────────────────────────────────────────────────────────
    {
        let vol_row = make_row("Volume", "Sound level (0% – 200%)");

        let vol_box = GBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .valign(Align::Center)
            .build();

        let slider = Scale::builder()
            .orientation(Orientation::Horizontal)
            .adjustment(&gtk4::Adjustment::new(
                config.borrow().volume as f64 * 100.0, // value
                0.0,   // lower
                200.0, // upper
                1.0,   // step
                10.0,  // page
                0.0,   // page_size
            ))
            .width_request(160)
            .draw_value(false)
            .build();

        slider.add_mark(0.0,   gtk4::PositionType::Bottom, None);
        slider.add_mark(100.0, gtk4::PositionType::Bottom, Some("100%"));
        slider.add_mark(200.0, gtk4::PositionType::Bottom, None);

        let vol_label = Label::builder()
            .label(&format!("{:.0}%", config.borrow().volume * 100.0))
            .width_chars(5)
            .build();

        let cfg = Rc::clone(&config);
        let cfg_path2 = cfg_path.clone();
        let vol_label2 = vol_label.clone();
        slider.connect_value_changed(move |s| {
            let v = s.value() / 100.0;
            cfg.borrow_mut().volume = v as f32;
            vol_label2.set_label(&format!("{:.0}%", s.value()));
            save_config(&cfg, &cfg_path2);
            restart_daemon();
        });

        vol_box.append(&slider);
        vol_box.append(&vol_label);
        vol_row.append(&vol_box);
        content.append(&vol_row);
    }

    content.append(&Separator::new(Orientation::Horizontal));

    // ── Play on press / release ───────────────────────────────────────────────
    {
        let row = make_row("Sound on Key Press", "Play sound when key is pressed down");
        let sw = Switch::builder()
            .active(config.borrow().play_on_press)
            .valign(Align::Center)
            .build();
        let cfg = Rc::clone(&config);
        let cfg_path2 = cfg_path.clone();
        sw.connect_active_notify(move |s| {
            cfg.borrow_mut().play_on_press = s.is_active();
            save_config(&cfg, &cfg_path2);
            restart_daemon();
        });
        row.append(&sw);
        content.append(&row);
    }

    {
        let row = make_row("Sound on Key Release", "Also play sound on key release");
        let sw = Switch::builder()
            .active(config.borrow().play_on_release)
            .valign(Align::Center)
            .build();
        let cfg = Rc::clone(&config);
        let cfg_path2 = cfg_path.clone();
        sw.connect_active_notify(move |s| {
            cfg.borrow_mut().play_on_release = s.is_active();
            save_config(&cfg, &cfg_path2);
            restart_daemon();
        });
        row.append(&sw);
        content.append(&row);
    }

    {
        let row = make_row("Randomize Pitch", "Slightly vary pitch for a natural feel");
        let sw = Switch::builder()
            .active(config.borrow().randomize_pitch)
            .valign(Align::Center)
            .build();
        let cfg = Rc::clone(&config);
        let cfg_path2 = cfg_path.clone();
        sw.connect_active_notify(move |s| {
            cfg.borrow_mut().randomize_pitch = s.is_active();
            save_config(&cfg, &cfg_path2);
            restart_daemon();
        });
        row.append(&sw);
        content.append(&row);
    }

    {
        let row = make_row("Per-Key-Group Sounds", "Space/Enter sound deeper than letters");
        let sw = Switch::builder()
            .active(config.borrow().per_key_group_sounds)
            .valign(Align::Center)
            .build();
        let cfg = Rc::clone(&config);
        let cfg_path2 = cfg_path.clone();
        sw.connect_active_notify(move |s| {
            cfg.borrow_mut().per_key_group_sounds = s.is_active();
            save_config(&cfg, &cfg_path2);
            restart_daemon();
        });
        row.append(&sw);
        content.append(&row);
    }

    content.append(&Separator::new(Orientation::Horizontal));

    // ── Preview button ────────────────────────────────────────────────────────
    {
        let preview_box = GBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .halign(Align::Center)
            .margin_top(4)
            .build();

        let btn = Button::builder()
            .label("▶  Preview Sound")
            .build();
        btn.add_css_class("suggested-action");
        btn.add_css_class("pill");

        let cfg = Rc::clone(&config);
        btn.connect_clicked(move |_| {
            play_preview(&cfg.borrow());
        });

        preview_box.append(&btn);
        content.append(&preview_box);
    }

    // ── Status bar ────────────────────────────────────────────────────────────
    {
        let status = Label::builder()
            .halign(Align::Center)
            .margin_top(4)
            .build();
        status.add_css_class("dim-label");

        let running = is_daemon_running();
        if running {
            status.set_label("● Daemon is running");
        } else {
            status.set_label("○ Daemon is not running");
        }
        content.append(&status);
    }

    root.append(&content);

    // ── CSS ───────────────────────────────────────────────────────────────────
    let css = gtk4::CssProvider::new();
    css.load_from_data(
        "
        window { background-color: @window_bg_color; }
        .title-1 { font-size: 22px; font-weight: bold; }
        .dim-label { opacity: 0.6; }
        .pill { border-radius: 9999px; padding: 8px 24px; }
        "
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.set_child(Some(&root));
    window.present();
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// A two-column settings row: label on left, widget on right
fn make_row(title: &str, subtitle: &str) -> GBox {
    let row = GBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let label_box = GBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .valign(Align::Center)
        .build();

    let title_lbl = Label::builder()
        .label(title)
        .halign(Align::Start)
        .build();

    let sub_lbl = Label::builder()
        .label(subtitle)
        .halign(Align::Start)
        .build();
    sub_lbl.add_css_class("dim-label");

    label_box.append(&title_lbl);
    label_box.append(&sub_lbl);
    row.append(&label_box);
    row
}

fn save_config(cfg: &Rc<RefCell<Config>>, path: &std::path::Path) {
    if let Err(e) = cfg.borrow().save(path) {
        eprintln!("Failed to save config: {e}");
    }
}

fn restart_daemon() {
    let _ = Command::new("systemctl")
        .args(["--user", "restart", "cosmic-keyboard-sounds"])
        .output();
}

fn is_daemon_running() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "cosmic-keyboard-sounds"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn play_preview(config: &Config) {
    use cosmic_keyboard_sounds::sounds;
    use std::io::Cursor;
    use rodio::{Decoder, DeviceSinkBuilder, Player};

    // Clone the needed values so the thread owns them
    let wav = match config.theme {
        Theme::CherryMxBlue  => sounds::cherry_mx_blue_press(12345, 1.0).data,
        Theme::CherryMxBrown => sounds::cherry_mx_brown_press(12345, 1.0).data,
        Theme::CherryMxRed   => sounds::cherry_mx_red_press(12345, 1.0).data,
        Theme::Typewriter    => sounds::typewriter_press(12345, 1.0).data,
    };
    let volume = config.volume;

    std::thread::spawn(move || {
        let Ok(sink) = DeviceSinkBuilder::open_default_sink() else { return };
        let cursor = Cursor::new(wav);
        let Ok(decoder) = Decoder::new(cursor) else { return };
        let player = Player::connect_new(sink.mixer());
        player.set_volume(volume as rodio::Float);
        player.append(decoder);
        player.play();
        player.sleep_until_end();
    });
}

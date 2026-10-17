# 🎹 cosmic-keyboard-sounds

> Mechanical keyboard sound effects for Linux — built for the [COSMIC desktop](https://system76.com/cosmic).

Every keystroke plays a satisfying mechanical switch sound, generated in real time with **no external sound files required**.

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-Linux%20%28Wayland%29-green)

---

## ✨ Features

- 🔊 **Procedurally generated sounds** — no audio files needed, everything synthesised in Rust
- ⌨️ **4 mechanical switch themes** — Cherry MX Blue, Brown, Red, Typewriter
- 🎯 **Per-key-group sounds** — Space and Enter sound deeper than regular letter keys
- 🎲 **Randomised pitch** — slight variation per keystroke for a natural feel
- 🖥️ **GTK4 GUI** — easy settings without touching config files
- ⚙️ **CLI** — change theme and volume from the terminal
- 🔄 **Systemd user service** — starts automatically on every login
- 🐧 **Wayland-native** — reads `/dev/input` directly, no X11 needed

---

## 🎵 Sound Themes

| Theme | Description |
|-------|-------------|
| `cherry-mx-blue` | 🔵 Clicky tactile — sharp click + bottom-out thud |
| `cherry-mx-brown` | 🟤 Soft tactile bump — no click, satisfying thud |
| `cherry-mx-red` | 🔴 Silent linear — soft thud only |
| `typewriter` | 🖊️ Vintage typewriter — heavy clack + metallic ring |

---

## 🚀 Installation

### Requirements

- Linux with Wayland
- Rust 1.70+
- PipeWire or PulseAudio
- GTK4 (for GUI only)

```bash
# Install system dependencies (Ubuntu/Pop!_OS/Debian)
sudo apt install libgtk-4-dev libasound2-dev pkg-config

# Add yourself to the input group (required once, then re-login)
sudo usermod -aG input $USER
```

### Build & Install

```bash
git clone https://github.com/YOUR_USERNAME/cosmic-keyboard-sounds
cd cosmic-keyboard-sounds

# Build
cargo build --release

# Install binaries
cp target/release/cosmic-keyboard-sounds ~/.local/bin/
cp target/release/cosmic-keyboard-sounds-gui ~/.local/bin/

# Install systemd service (auto-start on login)
cosmic-keyboard-sounds install
systemctl --user start cosmic-keyboard-sounds
```

---

## 🖥️ GUI

```bash
cosmic-keyboard-sounds-gui
```

Or search for **"Keyboard Sounds"** in your app launcher.

The GUI lets you:
- Switch themes with a dropdown
- Adjust volume with a slider
- Toggle press/release sounds
- Preview the current sound instantly

---

## ⚙️ CLI

```bash
# List available themes
cosmic-keyboard-sounds themes

# Change theme
cosmic-keyboard-sounds theme typewriter

# Set volume (0.0 – 2.0)
cosmic-keyboard-sounds volume 1.2

# Show current settings
cosmic-keyboard-sounds status

# Install / remove systemd service
cosmic-keyboard-sounds install
cosmic-keyboard-sounds uninstall
```

---

## 📁 Config File

Located at `~/.config/cosmic-keyboard-sounds/config.toml`:

```toml
enabled = true
theme = "cherry-mx-blue"
volume = 0.8
play_on_press = true
play_on_release = false
randomize_pitch = true
per_key_group_sounds = true
max_simultaneous = 8
```

---

## 🛠️ How It Works

Sounds are synthesised procedurally using layered techniques:

1. **Transient layer** — initial click/impact burst (band-limited noise + resonant sine)
2. **Body layer** — switch housing resonance
3. **Bottom-out layer** — key hitting the PCB/plate (low-frequency thud)
4. **Tail layer** — spring ping and housing ring-out

Each keypress gets a unique seed combining the key code and nanosecond timestamp, so no two keystrokes sound exactly the same.

---

## 🤝 Contributing

Issues and pull requests are welcome!

If you have a recorded mechanical keyboard sound pack you'd like to contribute (under a permissive license), please open an issue.

---

## 📄 License

MIT — see [LICENSE](LICENSE)

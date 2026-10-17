use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Available keyboard sound themes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum Theme {
    /// Cherry MX Blue — clicky tactile
    #[default]
    #[serde(rename = "cherry-mx-blue")]
    CherryMxBlue,
    /// Cherry MX Brown — soft tactile
    #[serde(rename = "cherry-mx-brown")]
    CherryMxBrown,
    /// Cherry MX Red — silent linear
    #[serde(rename = "cherry-mx-red")]
    CherryMxRed,
    /// Vintage typewriter
    #[serde(rename = "typewriter")]
    Typewriter,
}

impl FromStr for Theme {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cherry-mx-blue"  => Ok(Theme::CherryMxBlue),
            "cherry-mx-brown" => Ok(Theme::CherryMxBrown),
            "cherry-mx-red"   => Ok(Theme::CherryMxRed),
            "typewriter"      => Ok(Theme::Typewriter),
            other => Err(format!(
                "Unknown theme '{other}'. Use: cherry-mx-blue | cherry-mx-brown | cherry-mx-red | typewriter"
            )),
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::CherryMxBlue  => write!(f, "cherry-mx-blue (Clicky)"),
            Theme::CherryMxBrown => write!(f, "cherry-mx-brown (Tactile)"),
            Theme::CherryMxRed   => write!(f, "cherry-mx-red (Silent)"),
            Theme::Typewriter    => write!(f, "typewriter (Vintage)"),
        }
    }
}

/// Which "group" a key belongs to — each group gets a slightly different sound
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyGroup {
    /// Regular letter keys (A-Z)
    Letter,
    /// Number row (0-9) and symbols
    Number,
    /// Space bar — longest key, deepest sound
    Space,
    /// Enter / Return — satisfying clunk
    Enter,
    /// Backspace — slightly different from Enter
    Backspace,
    /// Modifier keys: Shift, Ctrl, Alt, Meta
    Modifier,
    /// Function keys F1-F12
    Function,
    /// Everything else: arrows, ins, del, etc.
    Other,
}

impl KeyGroup {
    /// Classify an evdev key code into a group
    pub fn from_key_code(code: u16) -> Self {
        match code {
            // Letters: A-Z (evdev codes 16-50, not all sequential)
            16..=25  => Self::Letter, // Q W E R T Y U I O P
            30..=38  => Self::Letter, // A S D F G H J K L
            44..=50  => Self::Letter, // Z X C V B N M
            // Number row
            2..=13   => Self::Number, // 1-0 and -=
            // Space
            57       => Self::Space,
            // Enter
            28       => Self::Enter,
            // Backspace
            14       => Self::Backspace,
            // Modifiers: L/R Shift, Ctrl, Alt, Meta, CapsLock
            29 | 42 | 54 | 56 | 97 | 100 | 125 | 126 | 58 => Self::Modifier,
            // Function keys F1-F12
            59..=68 | 87 | 88 => Self::Function,
            // Everything else
            _ => Self::Other,
        }
    }

    /// Pitch multiplier: space is lower, modifiers are mid, etc.
    pub fn pitch_factor(self) -> f32 {
        match self {
            Self::Space    => 0.72,  // deeper
            Self::Enter    => 0.85,  // slightly deeper
            Self::Backspace => 0.90,
            Self::Modifier  => 0.95,
            Self::Letter    => 1.00,  // baseline
            Self::Number    => 1.05,
            Self::Function  => 1.15,  // higher pitched (lighter keys)
            Self::Other     => 1.00,
        }
    }

    /// Volume factor: space is louder, function keys quieter
    pub fn volume_factor(self) -> f32 {
        match self {
            Self::Space    => 1.20,
            Self::Enter    => 1.15,
            Self::Backspace => 1.05,
            Self::Modifier  => 0.85,
            Self::Letter    => 1.00,
            Self::Number    => 0.95,
            Self::Function  => 0.75,
            Self::Other     => 0.90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub enabled: bool,
    pub theme: Theme,
    /// Volume 0.0–2.0
    pub volume: f32,
    pub play_on_press: bool,
    pub play_on_release: bool,
    /// Max simultaneous sounds
    pub max_simultaneous: usize,
    /// Randomize pitch ±5% for naturalness
    pub randomize_pitch: bool,
    /// Per-key-group sounds (letter sounds different from space, enter, etc.)
    pub per_key_group_sounds: bool,
    /// Only react to these key codes (empty = all keys)
    pub key_filter: Vec<u16>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: Theme::CherryMxBlue,
            volume: 0.8,
            play_on_press: true,
            play_on_release: false,
            max_simultaneous: 8,
            randomize_pitch: true,
            per_key_group_sounds: true,
            key_filter: vec![],
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let header = "# cosmic-keyboard-sounds config\n\
                      # Themes: cherry-mx-blue | cherry-mx-brown | cherry-mx-red | typewriter\n\
                      # Volume: 0.0 (silent) – 2.0 (very loud)\n\n";
        let body = toml::to_string_pretty(self)?;
        std::fs::write(path, format!("{header}{body}"))?;
        Ok(())
    }
}

pub fn config_path() -> PathBuf {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/root"));
    let dir = home.join(".config").join("cosmic-keyboard-sounds");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("config.toml")
}

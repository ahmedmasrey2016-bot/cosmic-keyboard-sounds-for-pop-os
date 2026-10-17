# Contributing to cosmic-keyboard-sounds

Thanks for your interest! Here's how to get started.

## Reporting Bugs

Open an issue with:
- Your OS and desktop environment
- The command you ran and the error output
- Output of `cosmic-keyboard-sounds status`

## Suggesting Features

Open an issue with the `enhancement` label. Common ideas that would be welcome:
- New sound themes
- Support for recorded sound packs (.ogg files per key)
- AppIndicator / system tray icon
- Per-application sound profiles

## Submitting Code

1. Fork the repo
2. Create a branch: `git checkout -b my-feature`
3. Make your changes
4. Run `cargo clippy` and fix any warnings
5. Open a pull request

## Adding a New Theme

1. Add a variant to `Theme` enum in `src/config.rs`
2. Add the synthesis functions in `src/sounds.rs`
3. Wire it up in `src/audio.rs` match blocks
4. Add it to the CLI themes list in `src/main.rs`
5. Add it to the GUI dropdown in `src/gui/app.rs`

## Code Style

- `cargo fmt` before committing
- Keep functions small and focused
- Comments in English

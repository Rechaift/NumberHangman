# Number Hangman
A Linux terminal number guessing + hangman game written in Rust.

### Features
- Multiple difficulty levels
- Hot/Cold hints
- Odd/Even hints
- Hangman-style attempt indicator
- Keyboard and mouse interaction
- Resizable terminal interface
- Multiple scenes and help pages

### Build
```bash
rustc -C opt-level=3 -C lto=fat -C strip=symbols main.rs -o numberhangman
```

### License
Apache License 2.0.
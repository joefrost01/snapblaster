# Snap-Blaster

**Snap-Blaster** is a live performance and production tool that lets you play your mix, effects, and sonic states like an instrument. Designed for genres like techno, drum & bass, ambient, and experimental electronica, Snap-Blaster turns your Launchpad (or any 8×8 grid) into a bank of programmable MIDI CC scenes.

It's like having 64 mix engineers, all ready to push faders, morph FX, or slam chain selectors the moment you trigger them — quantized and playable, in sync with your DAW.

## Key Features

- **64 pads, 64 CCs per pad** — instant scene recall for mix, FX, macros, or synth parameters
- **Morphing control** — define start/end values over time (e.g., "filter cutoff from 30 to 100 over 4 bars")
- **Ableton Link Sync** — CCs can be fired or morphed in time with your DAW
- **MIDI-first** — works with any DAW or hardware that responds to CCs (Ableton, Logic, Bitwig, Reason, MPC, synths)
- **No-op support** — selectively ignore CCs in scenes for partial state transitions
- **JSON-powered snapshots** — easy to hand-edit, version, and share
- **Launchpad-ready** — plug and play on grid controllers

## Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/snap-blaster.git
cd snap-blaster

# Build the project
cargo build --release

# Run the application
cargo tauri dev
```

## License

MIT — free for everyone, forever.

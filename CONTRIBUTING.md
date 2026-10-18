# Contributing to openbf

Thanks for wanting to help. Every kind of contribution counts: bug reports, feature ideas,
playtesting, documentation and code.

## Issues

- **Bugs:** what you did, what you expected, what happened instead, the map and mode, and a
  screenshot or a short clip if you can. Say which commit you built from (`git log -1 --oneline`).
- **Features and ideas:** describe what you want and why. Check `docs/loop/cards/` first; the card
  may already exist, and you can comment on it in your issue.

## Pull requests

1. Fork the repo and branch from `main`.
2. Build and test from `engine/`:
   ```bash
   cargo build --release
   ./ci.sh          # fmt, clippy -D warnings, build, the test suite
   ```
   Tests that need the baked game data skip themselves until you have run `scripts/pack.py` once.
3. Keep a pull request to one change, and say in its description what it changes and how you
   checked it.

House rules (from `PLAN.md` and `engine/CRATES.md`):
- **Never commit game content:** no textures, models, sounds, extracted scripts or disc files.
  Everything comes from the player's own discs at build time.
- **The simulation (`engine/crates/core`) is deterministic:** no wall clock, no OS randomness and
  no host floating-point library calls (use `core::fmath`).
- **Behaviour that copies the original game cites where it comes from** (a note in `analysis/`).
  A value that is a guess is marked `PROVISIONAL`.
- Source files stay under 1,500 lines.

## How the main line is developed

Most of the code is written by AI agents in a loop (`docs/loop/`), with every change gated by the
test suite and checked against the original game running in the PCSX2 emulator. That loop runs on a
private copy and is published here in batches, so a pull request may need a small rebase when a
batch lands. Human contributions are just as welcome: the loop's cards are open to anyone.

## License

By contributing you agree that your contribution is released under the MIT license (`LICENSE`).

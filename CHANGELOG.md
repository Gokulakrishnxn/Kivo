# Changelog

All notable changes to Kivo are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Release notes for tagged versions are drafted automatically from Conventional Commit
messages — see [CONTRIBUTING.md](CONTRIBUTING.md).

## [Unreleased]

### Added — Milestone 2: the macOS application

- Working menu-bar app: hold a key, speak, and cleaned text is inserted at the cursor.
- Cleanup engine with 33 tests, including a golden test reproducing the worked
  example from the PRD and the landing page.
- On-device recognition via NVIDIA Parakeet TDT 0.6B through sherpa-onnx.
- Lock-free SPSC ring buffer for capture; the microphone opens on key-down and
  closes on key-up so the OS indicator stays honest.
- Onboarding: five steps that explain each permission *before* triggering it.
- In-app model download, SHA-256 verified against a pinned digest, refusing to
  install on mismatch. Mirrored on this repository's releases.
- Persistent, searchable history in SQLite (mode 0600). Turning it off deletes the file.
- Vocabulary that biases recognition (hotwords) plus post-hoc replacements.
- Double-tap to latch hands-free; Esc discards a recording.
- Input device selection with a fallback to the system default.
- Overlay follows the pointer; app follows the OS light/dark theme.
- Live network panel that records actual outbound requests rather than asserting none.

### Fixed

- `NSMicrophoneUsageDescription` was missing, which would have made macOS terminate
  Kivo the instant it opened an input device.
- The binary carried no `LC_RPATH`, so the first bundle died at launch unable to
  load ONNX Runtime.
- `.gitignore`'s `Icon?` rule swallowed the app icon directory on case-insensitive
  filesystems.

### Known gaps

- Not signed or notarised, and there is no Windows build.
- History is not encrypted at rest, contrary to DEL-6.
- Word-level timestamps are unavailable through the current binding.
- Measured latency and memory both exceed the budgets in PRD §9.5.


### Changed — brand v2.0: cream parchment

- **Replaced the monochrome brand system entirely.** Kivo now looks like a storybook
  spread on cream parchment: warm paper surfaces, inset hairline borders instead of
  shadows, and a cast of hand-drawn characters carrying the personality. The v1
  black-and-white system is gone; only the geometry of the mark carried forward.
- The mark's three waveform bars are now sky, sun and ember, resolving into charcoal
  text lines — what you say arrives bright and messy, what Kivo returns is settled.
- Added five mascots (mic, bubble, page, caret, wave) and ten punctuation confetti
  shapes. The confetti is punctuation because punctuation is what Kivo adds.
- Display type is now Bricolage Grotesque at 500; Inter carries everything else.
- Marketing surfaces are light only. The application still follows the OS theme.
- Rebuilt the landing page and brand kit page on the new system, and rewrote
  [brand/BRAND.md](brand/BRAND.md), the design rules in [CLAUDE.md](CLAUDE.md) and the
  README design section to match.
- Regenerated the architecture diagrams in the new palette.
- CI now verifies the JSON and CSS token sources agree under the new structure, and that
  the `web/assets/` copies of the tokens and illustrations have not drifted.


### Added — Milestone 0: Foundation

- Product requirements document ([PRD.md](PRD.md)) covering the problem, users,
  principles, requirements, technical design, budgets, milestones and risks.
- System architecture ([docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)) with light and dark
  diagrams of the five-stage on-device pipeline.
- Complete brand identity: the waveform-to-text mark, a drawn geometric logotype,
  lockups, app icons and a theme-aware favicon.
- Design system — semantic tokens, type scale, spacing, radius, elevation and motion —
  published as both CSS custom properties and W3C design tokens. *(Superseded by
  brand v2.0 above.)*
- Brand guidelines ([brand/BRAND.md](brand/BRAND.md)) and an interactive brand kit page.
- Marketing website with a worked dictation demo. *(Redesigned in brand v2.0.)*
- Contribution guide, code of conduct, security policy and licence boundary.
- GitHub automation: CI, CodeQL, OpenSSF Scorecard, Dependabot, release drafter,
  labeler, stale bot, first-interaction greetings and Pages deployment.

### Notes

- Kivo is at Milestone 0. The application scaffold arrives at Milestone 2; see
  [PRD.md §11](PRD.md) for the delivery plan.

[Unreleased]: https://github.com/Gokulakrishnxn/Kivo/commits/main

# Changelog

All notable changes to Kivo are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Release notes for tagged versions are drafted automatically from Conventional Commit
messages — see [CONTRIBUTING.md](CONTRIBUTING.md).

## [Unreleased]

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

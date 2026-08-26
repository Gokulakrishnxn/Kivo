# Contributing to Kivo

Kivo is free and open source software. Contributions are genuinely welcome — including
your first one, including a typo fix, including "this paragraph confused me".

This guide is longer than it needs to be for a small change. **If you are fixing a typo
or a broken link, just open a pull request.** Everything below is for anything larger.

---

## Table of contents

- [The short version](#the-short-version)
- [Ways to help that are not code](#ways-to-help-that-are-not-code)
- [Before you write code](#before-you-write-code)
- [Getting set up](#getting-set-up)
- [How Kivo is organised](#how-kivo-is-organised)
- [Making a change](#making-a-change)
- [Commit messages](#commit-messages)
- [Pull requests](#pull-requests)
- [Design and brand changes](#design-and-brand-changes)
- [Reporting bugs](#reporting-bugs)
- [Privacy rules for contributors](#privacy-rules-for-contributors)
- [Licensing your contribution](#licensing-your-contribution)
- [Getting help](#getting-help)

---

## The short version

1. Open an issue before writing anything substantial, so you do not build something
   that gets turned down for a reason you could not have known.
2. Fork, branch from `main`, make the change.
3. Keep the pull request focused on one thing.
4. Say how you verified it, on which operating system.
5. Never include audio, transcripts or personal data.

---

## Ways to help that are not code

These are not consolation prizes. For a project like Kivo they are often worth more
than a patch.

- **Report an accuracy problem.** What you said, what Kivo typed, your OS and hardware.
  This is the highest-value bug report in the project.
- **Report an app where insertion fails.** Kivo has to put text into applications it
  does not control, and every one of them behaves differently. Finding a broken one is
  a real contribution — see the [architecture notes on delivery](docs/ARCHITECTURE.md).
- **Improve the documentation.** If something here confused you, that is a defect.
- **Test a release candidate** on hardware the maintainer does not own.
- **Answer a question** in Discussions.
- **Translate**, once the multilingual work lands.

## Before you write code

**Open an issue first for anything beyond a small fix.** This is not bureaucracy —
Kivo has a deliberately small surface, and [PRD.md §4](PRD.md) lists things that will
be declined no matter how well they are implemented. Nobody enjoys that conversation
after the work is done.

Good candidates to pick up without asking:

- Anything labelled [`good first issue`](https://github.com/Gokulakrishnxn/Kivo/labels/good%20first%20issue) or [`help wanted`](https://github.com/Gokulakrishnxn/Kivo/labels/help%20wanted)
- Documentation, tests, accessibility fixes
- Bugs with a clear reproduction already in the issue

Say in the issue that you are taking it, so two people do not do the same work.

## Getting set up

> **Milestone note.** Kivo is at Milestone 0 — the repository currently holds the
> product definition, the architecture, the brand system and the website. The
> application scaffold lands at Milestone 2 ([PRD.md §11](PRD.md)). Until then, the
> contribution surface is docs, brand, website and design.

**To work on the website or the brand system**, no toolchain is needed:

```bash
git clone https://github.com/Gokulakrishnxn/Kivo.git
cd Kivo
python3 -m http.server 8000 --directory web
```

Then open <http://localhost:8000>. The brand kit page is at `/brand.html`.

**When the application lands**, you will need Rust (stable), Node 20+, pnpm, and the
Tauri v2 prerequisites for your platform. The README will carry the exact commands at
that point rather than a promise here.

## How Kivo is organised

| Path | What lives there | Read first |
|---|---|---|
| `brand/` | Logo, design tokens, guidelines | [brand/BRAND.md](brand/BRAND.md) |
| `web/` | The website and the brand kit page | — |
| `docs/` | Architecture and diagrams | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) |
| `.github/` | CI, bots, templates | — |
| `src/` | Interface (React + TypeScript) | *from M2* |
| `src-tauri/` | Core (Rust) | *from M2* |

[PRD.md](PRD.md) is the product definition — requirements, budgets, non-goals and open
questions. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) explains why the boundaries are
where they are. Both are worth ten minutes before a substantial change.

## Making a change

```bash
git checkout -b your-branch-name
```

Branch names: `fix/clipboard-restore`, `feat/custom-vocabulary`, `docs/onboarding`,
`brand/dark-theme-tokens`. Anything descriptive is fine.

**Principles that will come up in review:**

- **Match the surrounding code.** Kivo does not need your preferred style; it needs one
  consistent style.
- **Never hard-code a colour, size, radius or duration.** Use the tokens in
  `brand/tokens/`. This is checked in CI.
- **Both themes, always.** Any interface change is reviewed in light and dark.
- **Accessibility is correctness.** Keyboard reachable, focus visible, labelled for
  screen readers. Not a follow-up ticket.
- **Respect the latency budget.** Anything in the dictation path is subject to
  [PRD.md §9.5](PRD.md). If your change costs milliseconds there, say so in the PR.
- **Do not add a setting casually.** Every preference is maintained forever. The bar is
  that its absence breaks a real workflow.

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/). The release notes are
generated from them, so the prefix genuinely matters.

```
feat(clean): collapse repeated-word stutters
fix(inject): restore clipboard contents after paste fallback
docs(prd): clarify the tiered cleanup rationale
brand(tokens): add compact-mark minimum sizes
chore(ci): pin actions to major versions
```

Types: `feat` · `fix` · `docs` · `brand` · `design` · `refactor` · `perf` · `test` ·
`chore` · `ci`.

Write the subject in the imperative — "add", not "added" — keep it under 72 characters,
and put the *why* in the body when it is not obvious. A commit that explains its
reasoning is a gift to whoever bisects it in two years.

## Pull requests

Open it against `main`. The template asks for what changed, why, and how you verified
it — please fill it in properly; "tested locally" tells a reviewer nothing.

What happens next:

1. Bots label the PR and run CI (repository health, SVG and JSON validity, token
   consistency, link checking, HTML sanity, CodeQL).
2. A maintainer reviews it personally. Expect a few days.
3. Review comments are about the code, not about you. If one reads harshly, it was
   written quickly, not unkindly — say so and it will be rephrased.
4. Merged with a squash, into a release draft.

**Keep it focused.** A PR that fixes a bug *and* reformats a file *and* renames a
variable is three PRs wearing a coat, and it will take far longer to review than the
three would have separately.

**Draft PRs are welcome** for work in progress or for a direction check before you go
further.

## Design and brand changes

Interface and brand work is held to the same standard as code, against
[brand/BRAND.md](brand/BRAND.md).

- **Include screenshots of both themes.** A PR that changes pixels without showing them
  cannot be reviewed.
- **Kivo is monochrome.** There is no accent colour, and introducing one is a brand
  change requiring a discussion first, not a design decision inside a PR.
- **Hairlines, not shadows.** Only the overlay and menus may cast a shadow.
- **The logo is not open for redesign** through a pull request. Improvements to how it
  is *applied* very much are.
- **Changing a token changes everything downstream.** Say what you audited.

## Reporting bugs

Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.yml). For accuracy
problems specifically, the useful report contains: what you said, what Kivo typed, your
OS and version, your hardware, your microphone, and the app you were dictating into.

**Do not open a public issue for a security or privacy vulnerability.** See
[SECURITY.md](SECURITY.md).

## Privacy rules for contributors

Kivo handles people's speech. These are not negotiable, and they apply to issues, pull
requests, discussions and commits alike.

1. **Never include real audio.** Not as an attachment, not as a link, not in a test
   fixture. Use synthetic or public-domain speech.
2. **Never include a real transcript** containing anything you would not publish.
   Redact freely — a redacted report is far more useful than no report.
3. **Redact screenshots.** Dictation happens inside people's actual work.
4. **Never add telemetry, analytics, crash reporting or any outbound request** without
   an explicit prior discussion and a maintainer's agreement. This is a promise Kivo
   has made to its users ([PRD.md §8.5](PRD.md)), and a PR that quietly breaks it will
   be closed rather than revised.

## Licensing your contribution

By contributing you agree that your contribution is licensed under the
**MIT Licence**, copyright **Gokulakrishnan**, matching [LICENSE](LICENSE).

You keep authorship — git history is permanent and you will appear in the contributor
list and release notes. There is no CLA to sign.

Note the boundary in [NOTICE](NOTICE): the MIT licence covers the *code*. The Kivo name
and logo do not fall under it. Contributing does not grant you rights to the brand, and
it does not take yours away either.

## Getting help

- **Questions and ideas** → [Discussions](https://github.com/Gokulakrishnxn/Kivo/discussions)
- **Bugs** → [Issues](https://github.com/Gokulakrishnxn/Kivo/issues)
- **Vulnerabilities** → [SECURITY.md](SECURITY.md), privately
- **Stuck on a PR** → comment on it. Asking is faster than guessing, and nobody minds.

Everyone taking part is expected to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

---

Thank you for being here. A project like this is only meaningfully open if people
other than its author can get into it — if any part of that was harder than it should
have been, that is a bug worth reporting too.

<sub>Kivo — Copyright © 2026 Gokulakrishnan. MIT licensed.</sub>

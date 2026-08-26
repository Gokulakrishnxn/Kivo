# CLAUDE.md — working rules for Kivo

Instructions for Claude Code (and any AI collaborator) working in this repository.
Human contributors should read [CONTRIBUTING.md](CONTRIBUTING.md) instead; this file
covers the things an agent needs stated explicitly.

---

## What Kivo is

A desktop dictation utility for **macOS and Windows**. Hold a key, speak, release, and
clean text appears at the cursor in whatever app is focused. Everything runs on-device.
No account, no subscription, no audio leaving the machine. MIT licensed, copyright
Gokulakrishnan.

Read [PRD.md](PRD.md) before any substantial change. Read
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) before touching the pipeline.

---

## Git workflow — commit and push

**Commit and push completed work to GitHub without waiting to be asked.**

This is the standing instruction for this repository. When a unit of work is finished
and verified, commit it and push it to `origin`. Do not leave finished work sitting
uncommitted in the working tree, and do not ask "shall I commit this?" for ordinary
changes.

The rules around that:

1. **Commit when a unit of work is complete**, not when the session ends. A commit
   should be one coherent change — a document written, a component built, a bug fixed.
   Several small, meaningful commits beat one large one.
2. **Push to `origin` after committing.** The remote is
   `https://github.com/Gokulakrishnxn/Kivo.git`, default branch `main`.
3. **Verify before committing.** Run whatever check applies — the site loads, the SVG
   parses, the JSON is valid, the tests pass. Never commit something you have not
   checked, and never report a commit as verified when the check failed.
4. **Use Conventional Commits.** The release notes are generated from them.
   `feat(clean): …` · `fix(inject): …` · `docs(prd): …` · `brand(tokens): …` ·
   `chore(ci): …`. Imperative mood, under 72 characters, body explaining *why* when it
   is not obvious.
5. **Never commit** audio files, transcripts, real user data, `.env` files, credentials,
   API keys, model weights, or build output. `.gitignore` covers the common cases; it is
   not a substitute for looking.
6. **Direct commits to `main` are fine** for solo maintainer work. Use a branch and a PR
   when the change is large enough to want review, or when the user asks for one.
7. **Stop and ask** before anything that rewrites shared history or discards work:
   `push --force`, `reset --hard` on pushed commits, `rebase` of pushed branches,
   deleting branches or tags, or changing repository visibility or settings.
8. **Attribute honestly.** Co-author trailers stay on commits an agent authored.

---

## Non-negotiables

These are promises Kivo has made to its users. Breaking one is not a design trade-off.

1. **No audio or transcripts leave the device.** Ever.
2. **No telemetry, analytics or crash reporting.** Not even anonymised, not even opt-in,
   without an explicit decision from the maintainer recorded in a release note.
3. **The only outbound requests** are the model download (SHA-256 pinned) and the
   opt-out update check. Adding a third requires a conversation first.
4. **Audio is never written to disk.** It lives in a ring buffer that is zeroed after use.
5. **The microphone opens on key-down and closes on key-up**, so the OS indicator tells
   the truth.

If a task appears to require breaking one of these, stop and say so rather than finding
a way around it. See [SECURITY.md](SECURITY.md) for the full list of properties.

---

## Design rules

The full system is in [brand/BRAND.md](brand/BRAND.md). The parts most easily got wrong:

- **Kivo is monochrome.** There is no accent colour. Emphasis comes from inversion and
  weight, never hue. Introducing a colour is a brand change, not a design decision.
- **Never hard-code a colour, size, radius or duration.** Everything comes from
  `brand/tokens/kivo-tokens.css`. CI checks that the CSS and JSON token sources agree.
- **Hairlines, not shadows.** A 1 px border is how surfaces separate. Only the dictation
  overlay and menus may cast a shadow, using the two defined shadow tokens.
- **Both themes, always.** Verify light and dark before calling an interface change done.
- **Ink is `#0A0A0A`, not `#000000`.** Pure black is reserved for the dark theme's inset.
- **Sentence case** for all headings and buttons. The product name is always **Kivo**.
- **The logo is not open for redesign.** Improving how it is applied is welcome.
- Typography: **Inter** for interface, **JetBrains Mono** for anything measured,
  versioned or machine-typed. Mono is used as a texture, deliberately.

## Writing rules

- Plain, specific, second person, active voice. Numbers over adjectives.
- **Never anthropomorphise Kivo.** It records, recognises and inserts. It does not
  listen, understand, think or help. The brand's credibility depends on this.
- No exclamation marks or emoji in product copy. Sparingly in README and templates, for
  navigation only.
- Own failures plainly: "Kivo could not insert text into this app, so it copied it to
  your clipboard instead."

---

## Engineering rules

- **Match the surrounding code.** Consistency beats personal preference.
- **Respect the latency budget** in [PRD.md §9.5](PRD.md) — 300 ms from key-up to text.
  If a change costs milliseconds in the dictation path, say so explicitly.
- **The audio callback and hotkey hook must never block.** No allocation, no locks, no
  logging in either. Being slow there is a correctness bug, not a performance one.
- **Recognition sits behind a trait.** Keep the `asr/` boundary clean so the model can
  be swapped without a rewrite.
- **Do not add a setting casually.** Every preference is maintained forever.
- **Accessibility is correctness.** Keyboard reachable, focus visible, screen-reader
  labelled, WCAG 2.2 AA. Not a follow-up ticket.
- **Do not add dependencies casually**, especially in the Rust core. Kivo is a resident
  process handling private audio; every dependency is attack surface.

## Repository map

| Path | Contents |
|---|---|
| `PRD.md` | Product definition — requirements, budgets, non-goals, milestones |
| `docs/ARCHITECTURE.md` | System design; `docs/diagrams/` holds the light and dark SVGs |
| `brand/BRAND.md` | Brand and design system |
| `brand/logo/`, `brand/tokens/` | Canonical assets and tokens |
| `web/` | Website (`index.html`) and brand kit page (`brand.html`) |
| `.github/` | CI, bots, templates |
| `src/`, `src-tauri/` | Interface and core — *arrive at Milestone 2* |

## Checks to run before committing

The website and brand assets have no build step. What CI checks, and what you should
check locally:

```bash
# every SVG parses
find . -name '*.svg' -not -path './node_modules/*' -exec python3 -c "import sys,xml.dom.minidom;xml.dom.minidom.parse(sys.argv[1])" {} \;

# every JSON parses
python3 -c "import json;json.load(open('brand/tokens/kivo-tokens.json'))"

# serve the site and look at it, in both themes
python3 -m http.server 8000 --directory web
```

Verify interface and website changes by actually rendering them, not by reading the
markup. Screenshot both themes.

## Current state

**Milestone 0 is complete** — product definition, architecture, brand system, website
and repository automation. The application scaffold arrives at **Milestone 2**. Until
then the working surface is docs, brand, website and design.

See [PRD.md §11](PRD.md) for the milestone plan and [CHANGELOG.md](CHANGELOG.md) for
what has landed.

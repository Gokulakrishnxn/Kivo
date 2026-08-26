# Kivo — Product Requirements Document

| | |
|---|---|
| **Product** | Kivo — turns voice into clean text |
| **Version** | 0.1 (pre-alpha) |
| **Status** | Draft for build |
| **Author** | Gokulakrishnan ([@Gokulakrishnxn](https://github.com/Gokulakrishnxn)) |
| **Last updated** | 26 August 2026 |
| **Licence** | Code: MIT © Gokulakrishnan. Brand: see [NOTICE](NOTICE) |

---

## 1. Summary

Kivo is a desktop dictation utility for macOS and Windows. You hold a key, you speak,
you let go, and clean text appears at your cursor in whatever application you were
already using.

"Clean" is the entire product. Existing dictation transcribes what you *said* —
including "um", false starts, and the sentence you abandoned halfway through. Kivo
delivers what you *meant*: punctuated, capitalised, de-filled, and formatted for the
context you are typing into.

Everything runs on the device. No account, no subscription, no audio leaving the
machine. The source is public and MIT licensed.

---

## 2. The problem

Typing is the bottleneck between having a thought and recording it. People speak at
roughly 130–150 words per minute and type at 40–60. That gap has been known for
decades, and dictation still has not displaced the keyboard. Three reasons:

1. **The output needs editing.** Raw transcription produces a wall of unpunctuated
   text littered with disfluencies. Cleaning it up costs back the time dictation saved.
2. **It does not live where you work.** Dictation tends to be trapped in one app, or
   behind a web page you must copy out of. It is not available at the cursor, everywhere.
3. **It is a privacy trade.** The accurate options stream your microphone to a server
   and ask for a subscription. For anyone handling client work, medical notes, legal
   drafts, or unreleased code, that is not a trade they can make.

Kivo exists because the third constraint recently stopped being a constraint.
On-device ASR models are now fast and accurate enough that the cloud is no longer
the price of quality.

## 3. Goals

**G1 — Faster than typing, honestly.** End-to-end, dictating a paragraph with Kivo
must beat typing it for a competent typist, *including* any correction the user has
to do afterwards. Time-to-usable-text is the metric, not word error rate.

**G2 — Clean by default.** The first output is the final output. The user should not
routinely need to fix punctuation, capitalisation, or filler words.

**G3 — Everywhere the cursor is.** One global hotkey, any application, text inserted
in place. No copy-paste dance, no separate window to work in.

**G4 — Provably private.** Audio never leaves the device, and a technically literate
user can verify that claim themselves in under five minutes.

**G5 — Genuinely open.** Full source, permissive licence, buildable from a clean
checkout with documented commands, and a contribution path that a first-timer can follow.

## 4. Non-goals

These are deliberate exclusions for v1. Each is a real thing people will ask for.

| Not doing | Why |
|---|---|
| Real-time meeting transcription / diarisation | Different product. Kivo is a *typing replacement*, not a recorder. |
| Voice commands ("open Safari", "delete that") | Voice control is an OS-level concern and a large accuracy risk surface. |
| Mobile apps (iOS, Android) | Both platforms already have competent system dictation; the gap is on desktop. |
| Cloud sync of transcripts | Contradicts G4. History stays local. |
| Accounts, licences, subscriptions | Contradicts G5. There is nothing to log in to. |
| Linux (v1) | Wanted, but text injection across X11/Wayland is a project of its own. Post-v1. |
| Live streaming partial results into the target app | Partials that rewrite themselves are visually disruptive in third-party apps. Kivo inserts once, on release. |

## 5. Users

**Primary — the high-volume writer at a keyboard.** Engineers writing PR descriptions
and commit messages, support staff answering tickets, founders answering email,
students taking notes. They type all day, they are fast, and they are looking for
leverage, not accessibility.

**Secondary — the privacy-bound professional.** Lawyers, clinicians, therapists,
journalists, and anyone under NDA. They *want* dictation and are structurally
prohibited from using the cloud versions. For this group, on-device is not a feature,
it is the entry requirement.

**Tertiary — the user with a physical need.** RSI, limited hand mobility, or fatigue.
Kivo is not marketed as assistive technology and will not claim clinical suitability,
but accessibility requirements are treated as correctness requirements, not nice-to-haves.

## 6. Product principles

1. **The keyboard is the model, not the microphone.** Kivo should feel like a key you
   press, not an assistant you talk to. No wake word, no conversation, no personality.
2. **Silence is the default state.** No sounds, no notifications, no dock bouncing.
   The only feedback is a small overlay while you are actually speaking.
3. **On-device unless the user says otherwise.** Any feature that requires the network
   is opt-in, clearly labelled, and off in a fresh install.
4. **Monochrome.** Kivo's interface is black and white. Emphasis comes from inversion
   and weight, never colour. See [brand/BRAND.md](brand/BRAND.md).
5. **A small surface, held.** Every setting added is a setting maintained forever.
   The bar for a new preference is that its absence breaks a real workflow.

---

## 7. The core experience

### 7.1 The loop

```
  ┌── hold Fn (or your chosen key) ──────────────────────────┐
  │                                                          │
  │   A small overlay appears near the cursor.               │
  │   A live waveform shows Kivo is hearing you.             │
  │                                                          │
  │   You speak.                                             │
  │                                                          │
  └── release ───────────────────────────────────────────────┘
                     ↓  target: under 300 ms
       Clean text is inserted at the cursor. Overlay fades.
```

There is no confirm step, no preview, no "insert" button. If the result is wrong,
Cmd/Ctrl+Z removes it in one undo, because Kivo inserts as a single atomic edit.

### 7.2 First run

The single hardest moment in this product is the first ninety seconds, because Kivo
needs two OS permissions that look alarming out of context (microphone, and
accessibility/input control), plus a model download of several hundred megabytes.

The onboarding sequence is therefore:

1. **What Kivo does** — one screen, one sentence, a Continue button.
2. **Microphone** — explain *before* triggering the OS prompt. Then trigger it.
3. **Model download** — show size, source, and the SHA-256 that will be verified.
   Download with a real progress bar. This is the only moment Kivo touches the network.
4. **Accessibility / input permission** — explain that this is how text reaches your
   cursor, and that Kivo reads nothing from the screen. Then trigger the OS prompt.
5. **Pick your key** — default suggested, immediately rebindable.
6. **Try it** — a text box in the onboarding window. The user dictates one sentence
   and sees it appear. This step is not skippable, because a user who has not
   succeeded once will not succeed later.

### 7.3 Modes

A mode changes only the cleanup layer, never the recognition. The user switches modes
from the tray, or binds a different hotkey per mode.

| Mode | Behaviour |
|---|---|
| **Standard** (default) | Sentence case, full punctuation, fillers removed. |
| **Message** | Shorter sentences, no trailing full stop on a single-line message, contractions kept. |
| **Email** | Paragraph breaks honoured, greeting and sign-off capitalised, more formal register. |
| **Notes** | Bullet points inferred from "next point", "new bullet". Fragments allowed. |
| **Code** | No sentence casing. Spoken symbols mapped (`open paren`, `underscore`). Identifiers left alone. |
| **Raw** | Recognition output verbatim. The escape hatch, and the debugging baseline. |

### 7.4 What "clean" means concretely

Given the spoken input:

> "um so i think we should uh push the release to friday — no wait, thursday — because
> the uh the QA cycle needs two full days"

**Raw ASR output** (Parakeet already restores punctuation and case):
> "Um, so I think we should uh push the release to Friday. No wait, Thursday, because
> the uh the QA cycle needs two full days."

**Kivo Standard output:**
> "I think we should push the release to Thursday, because the QA cycle needs two full days."

The cleanup layer did four things: removed fillers (`um`, `uh`), resolved the
self-correction (`Friday — no wait, Thursday`), collapsed the stutter (`the uh the`),
and dropped the throat-clearing opener (`so`). Only the first and third are safe to
do with rules; the second and fourth need the optional local LLM pass. This is why
cleanup is tiered — see §9.4.

---

## 8. Requirements

Priority: **P0** ships in v1.0 or v1.0 does not ship. **P1** is v1.x. **P2** is later.

### 8.1 Capture and trigger

| ID | Priority | Requirement |
|---|---|---|
| CAP-1 | P0 | A global hotkey works in every focused application, including full-screen apps. |
| CAP-2 | P0 | Push-to-talk: recording starts on key-down, ends on key-up. |
| CAP-3 | P0 | Hands-free toggle: a double-tap of the hotkey latches recording until tapped again. |
| CAP-4 | P0 | The hotkey is rebindable, including to a single modifier key (Fn, Right Cmd, Right Ctrl). |
| CAP-5 | P0 | Kivo opens the microphone only while recording, and releases it immediately after. The OS mic indicator must reflect this honestly. |
| CAP-6 | P0 | Input device is selectable; Kivo follows the system default if the chosen device disappears. |
| CAP-7 | P1 | Per-mode hotkeys. |
| CAP-8 | P1 | Recording is cancelled by Esc, discarding audio without inserting anything. |

### 8.2 Recognition

| ID | Priority | Requirement |
|---|---|---|
| ASR-1 | P0 | Recognition runs fully on-device with no network access. |
| ASR-2 | P0 | English (en-US, en-GB, en-IN) at a word error rate competitive with cloud dictation on clean near-field audio. |
| ASR-3 | P0 | Punctuation and capitalisation restored by the model. |
| ASR-4 | P0 | Hardware acceleration used where available, with an automatic CPU fallback that still meets the latency budget on a 4-core machine. |
| ASR-5 | P0 | Word-level timestamps retained internally (needed by the cleanup layer and future editing). |
| ASR-6 | P1 | Custom vocabulary — the user adds names, acronyms and jargon that bias recognition. |
| ASR-7 | P1 | Additional languages via the multilingual Parakeet variant, selectable per mode. |
| ASR-8 | P2 | Automatic language detection. |

### 8.3 Cleanup

| ID | Priority | Requirement |
|---|---|---|
| CLN-1 | P0 | Filler words and stutters removed (`um`, `uh`, `er`, repeated-word stutters). |
| CLN-2 | P0 | Spacing, quotes and dashes normalised; no double spaces; no space before punctuation. |
| CLN-3 | P0 | User-defined text replacements applied (spoken → written). |
| CLN-4 | P0 | Every cleanup stage is individually switchable, and Raw mode bypasses all of them. |
| CLN-5 | P1 | Optional local LLM polish for self-corrections, sentence repair and mode register. Off by default, model downloaded separately, clearly labelled as slower. |
| CLN-6 | P1 | Spoken formatting commands: "new line", "new paragraph", "bullet point". |
| CLN-7 | P2 | Learns replacements from corrections the user makes immediately after insertion. |

### 8.4 Delivery

| ID | Priority | Requirement |
|---|---|---|
| DEL-1 | P0 | Text is inserted at the cursor in the focused application as a single undoable edit. |
| DEL-2 | P0 | Where direct insertion is unavailable, Kivo falls back to clipboard-paste and **restores the previous clipboard contents** afterwards. |
| DEL-3 | P0 | Insertion respects an existing selection (replaces it), matching normal typing behaviour. |
| DEL-4 | P0 | A local history of the last N transcripts, searchable, with one-click re-copy. Default N = 100. |
| DEL-5 | P0 | History can be cleared entirely, and can be disabled so nothing is ever written to disk. |
| DEL-6 | P1 | History database encrypted at rest using an OS keychain-held key. |

### 8.5 Privacy and trust

| ID | Priority | Requirement |
|---|---|---|
| PRV-1 | P0 | Audio is never written to disk. It exists in a ring buffer and is zeroed after recognition. |
| PRV-2 | P0 | Zero telemetry. Not "anonymised telemetry" — none, with no opt-in to add later without an explicit release note. |
| PRV-3 | P0 | The only outbound requests are the model download and the update check. The update check is opt-out in settings. |
| PRV-4 | P0 | Settings shows a live network panel: what Kivo has contacted, when, and why. Empty is the normal state. |
| PRV-5 | P0 | Model artefacts verified against a pinned SHA-256 before being loaded. |
| PRV-6 | P1 | Releases are reproducible, or accompanied by signed build provenance (SLSA attestation). |

### 8.6 Interface

| ID | Priority | Requirement |
|---|---|---|
| UI-1 | P0 | Lives in the menu bar (macOS) / system tray (Windows). No dock or taskbar presence while idle. |
| UI-2 | P0 | The recording overlay appears near the caret, is never focusable, and never steals input. |
| UI-3 | P0 | Follows the system light/dark theme, and can be pinned to either. |
| UI-4 | P0 | Fully keyboard navigable. Meets WCAG 2.2 AA for contrast and focus visibility. |
| UI-5 | P0 | Screen-reader labels on every control (VoiceOver, Narrator). |
| UI-6 | P1 | The overlay's live waveform reflects real input level, so a dead microphone is visible instantly. |

### 8.7 Distribution

| ID | Priority | Requirement |
|---|---|---|
| DIS-1 | P0 | macOS: signed, notarised `.dmg`, universal binary (Apple Silicon + Intel). |
| DIS-2 | P0 | Windows: signed `.msi` and a portable `.exe`, x64 and ARM64. |
| DIS-3 | P0 | In-app updates, signed, with release notes shown before applying. |
| DIS-4 | P0 | A clean checkout builds with documented commands on both platforms. |
| DIS-5 | P1 | Homebrew cask (`brew install --cask kivo`) and WinGet (`winget install Kivo`). |

---

## 9. Technical design

### 9.1 Stack

| Layer | Choice | Why |
|---|---|---|
| **Shell** | Tauri v2 (Rust core) | A dictation tool is resident all day. Tauri gives a ~10 MB binary and ~40 MB idle RSS against Electron's ~150 MB+, uses the OS webview, and — decisively — lets the audio and injection paths live in native Rust rather than behind a JS bridge. |
| **Interface** | React 19 + TypeScript + Vite | Settings and onboarding are ordinary forms; the ecosystem and type safety matter more than raw render speed. The overlay is deliberately kept trivial. |
| **Styling** | Tailwind CSS v4, driven by `brand/tokens/kivo-tokens.css` | Tokens are the source of truth; Tailwind consumes them. No hard-coded colours anywhere. |
| **UI primitives** | Radix UI | Accessible-by-construction menus, dialogs and switches. UI-4 and UI-5 are much cheaper to satisfy with these than without. |
| **ASR model** | **NVIDIA Parakeet TDT 0.6B** | See §9.2. |
| **Inference** | ONNX Runtime via the `ort` crate; `sherpa-onnx` for the TDT decode loop | Runs the same exported model on CoreML, DirectML, CUDA and CPU with one code path. Avoids shipping a PyTorch runtime. |
| **Audio** | `cpal` capture, `rubato` resampling to 16 kHz mono | The standard cross-platform Rust audio stack, and the only one with mature CoreAudio *and* WASAPI backends. |
| **VAD** | Silero VAD (ONNX) | ~1 MB, negligible cost, and it lets Kivo trim silence before the expensive model sees the audio. |
| **LLM polish (opt-in)** | `llama.cpp` via `llama-cpp-rs`, a 1–3B instruct model in GGUF | Only runs when the user turns it on. Keeping it out-of-band means the default path stays fast. |
| **Storage** | SQLite via `rusqlite`; settings as TOML | One file, no server, trivially inspectable — which supports PRV-4's "verify it yourself". |
| **Keychain** | `keyring` crate | Holds the history encryption key (DEL-6). |
| **Build / release** | GitHub Actions; `tauri-action`; Apple notarytool; Azure Trusted Signing | Signed artefacts for both platforms from CI. |

### 9.2 Why Parakeet

NVIDIA's Parakeet TDT 0.6B is a FastConformer encoder with a Token-and-Duration
Transducer decoder, released under CC-BY-4.0 — a licence that permits commercial
redistribution, which most competitive open ASR weights do not.

It is the right fit for Kivo specifically because:

- **It is fast in the way dictation needs.** The TDT decoder predicts token durations
  and skips frames rather than stepping every frame. For short utterances the model
  finishes far faster than real time even on CPU, which is what makes the sub-300 ms
  budget in §9.5 plausible at all.
- **It emits punctuation and capitalisation natively.** No second model, no separate
  punctuation-restoration stage, no extra latency. Requirement ASR-3 is satisfied by
  the model rather than by us.
- **It emits word-level timestamps**, which the cleanup layer needs to detect stutters
  and self-corrections by timing rather than by string matching alone.
- **0.6B parameters is the right size.** Roughly 600 MB in fp32, ~300 MB quantised —
  large enough to be accurate, small enough that a one-time download is acceptable
  and that it fits comfortably in memory alongside the user's real work.
- **There is a multilingual sibling** (Parakeet TDT 0.6B v3, 25 European languages),
  which makes ASR-7 a model swap rather than a re-architecture.

> **Milestone 1 measurement — 26 August 2026. Parakeet misses the latency budget.**
>
> Measured with `cargo run --release --example transcribe`, Parakeet TDT 0.6B v2
> int8 via sherpa-onnx on the CoreML provider, Apple Silicon, 7.43 s of clean
> read speech:
>
> | | |
> |---|---|
> | Model load (once, at startup) | 6.5 s |
> | First inference, cold | 1453 ms |
> | **Warm median, 7 runs** | **663 ms** — 11.2× real time |
> | Cleanup (tiers 0–1) | 0.7 ms |
>
> Extrapolated to the ten-second utterance §9.5 budgets, recognition alone lands
> near **890 ms against a 300 ms budget** — roughly 3× over. Transcription quality
> was correct, including punctuation and capitalisation.
>
> This does not settle the question yet, and the model is not being swapped on one
> datapoint. Still to test before deciding: the CPU provider (CoreML may be falling
> back per-op and paying transfer costs for nothing), thread count, the non-quantised
> export, and a shorter utterance — a two-second dictation is the common case and
> may behave very differently from a 7.4 s read passage.
>
> If the budget still cannot be met, the options in order are: accept a higher budget
> and say so publicly, ship a smaller Parakeet variant, or fall back to a distilled
> Whisper at a known accuracy cost. The trait boundary in `asr/` exists precisely so
> that stays a contained change — it is already carrying a second implementation
> (`Unavailable`) today.
>
> Published leaderboard figures (~6 % WER) remain what they always were: read speech
> on a datacentre GPU, not a prediction of Kivo's numbers.

### 9.3 Pipeline

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full diagram and the
reasoning behind each boundary.

```
hotkey ──▶ capture ──▶ ring buffer ──▶ resample ──▶ VAD ──▶ Parakeet ──▶ cleanup ──▶ inject
                                                                             │
                                                                             └─▶ history
```

### 9.4 The cleanup tiers

Cleanup is tiered so that the default path never pays for the expensive tier.

- **Tier 0 — Normalisation.** Whitespace, quotes, dashes. Deterministic. Microseconds.
- **Tier 1 — Rules.** Filler removal, stutter collapse, user replacements, mode casing.
  Deterministic, unit-testable, auditable by the user. Sub-millisecond.
- **Tier 2 — Local LLM (opt-in, off by default).** Self-corrections, sentence repair,
  register. Adds 200–800 ms depending on the model and hardware, which is why it is a
  choice rather than a default.

A user who never opens settings gets Tiers 0 and 1, which handles the common case.
This tiering exists so that the honest answer to "is it fast?" does not depend on
which features happen to be enabled.

### 9.5 Performance budget

Measured from **hotkey release** to **text visible in the target application**, for a
ten-second utterance, on a base-model current-generation laptop.

| Stage | Budget |
|---|---|
| Finalise capture, drain buffer | 10 ms |
| Resample + VAD trim | 15 ms |
| Parakeet inference (accelerated) | 180 ms |
| Cleanup Tiers 0–1 | 5 ms |
| Text injection | 40 ms |
| **Total** | **≤ 300 ms** |

Also budgeted: **idle RSS under 150 MB** with the model resident, **idle CPU under
0.5 %**, and **cold start to first usable dictation under 2 s**. A dictation tool that
makes the machine feel slower will be uninstalled regardless of its accuracy.

### 9.6 Platform specifics

| Concern | macOS | Windows |
|---|---|---|
| Global hotkey | `CGEventTap` (needs Accessibility) | `RegisterHotKey` / low-level keyboard hook |
| Text insertion | `AXUIElement` set-value, else synthesised `CGEvent` keystrokes | UI Automation `TextPattern`, else `SendInput` |
| Audio | CoreAudio via `cpal` | WASAPI via `cpal` |
| Acceleration | CoreML EP (Neural Engine / GPU) | CUDA EP on NVIDIA, else DirectML, else CPU |
| Permissions | Microphone + Accessibility, both prompted with context first | Microphone; UIAccess only if required |
| Signing | Developer ID + notarytool | Azure Trusted Signing |
| Autostart | `SMAppService` | Registry `Run` key |

Text injection is the single largest source of platform-specific bugs in tools of this
kind: Electron apps, terminals, and Java applications each mishandle synthetic input
differently. **DEL-2's clipboard fallback is therefore not a degraded path — it is the
reliability guarantee**, and every release is tested against a fixed matrix of target
applications (native, Electron, browser, terminal, Office, IDE) on both platforms.

---

## 10. Success metrics

Kivo has no telemetry (PRV-2), so these are measured on an internal harness and
through voluntary reports, not collected from users. That is a deliberate cost.

**Quality gates — measured by us, per release, on a fixed corpus:**

| Metric | Target |
|---|---|
| Word error rate, clean near-field English | ≤ 8 % |
| Median hotkey-release → text latency | ≤ 300 ms |
| 95th percentile latency | ≤ 600 ms |
| Successful insertion across the app test matrix | 100 % (clipboard fallback counts) |
| Idle memory | < 150 MB |
| Crash-free sessions | > 99.5 % |

**Product signals — reviewed monthly:**

| Signal | Why it matters |
|---|---|
| Median edits per dictation, in our own daily use | The only honest measure of "clean". |
| Ratio of Raw-mode usage | High Raw usage means the cleanup layer is not trusted. |
| Time from install to first successful dictation | Onboarding is the biggest drop-off risk. |
| Issues tagged `accuracy` per 100 releases-downloaded | Proxy for real-world WER. |
| First-time contributors merged per quarter | Whether G5 is real or decorative. |

---

## 11. Milestones

| # | Milestone | Contents | Exit criterion |
|---|---|---|---|
| **M0** | Foundation | Repo, licence, brand, PRD, landing page, CI | *Complete* |
| **M1** | Spike | Parakeet exported to ONNX; latency + WER measured on both platforms | Numbers in §9.5 met, or model choice revised in writing |
| **M2** | Walking skeleton | Hotkey → capture → recognise → inject, macOS only, no interface | One end-to-end dictation into a real app |
| **M3** | Windows parity | Same skeleton on Windows | Same test passes on both |
| **M4** | Cleanup | Tiers 0–1, modes, replacements, full test suite | §7.4 example produces the Standard output |
| **M5** | Product | Onboarding, settings, overlay, history, tray | A stranger installs and succeeds unaided |
| **M6** | Hardening | Signing, notarisation, updater, app test matrix | Signed artefacts install cleanly on clean machines |
| **M7** | v1.0 | Docs, site, release | Public release |

---

## 12. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Parakeet misses the CPU latency budget on low-end hardware | Medium | High | M1 spike settles this before anything is built on top. `asr/` is a trait boundary; a distilled Whisper is the fallback. |
| Text injection fails in specific applications | **High** | High | Treated as expected, not exceptional. Clipboard fallback (DEL-2) plus a fixed per-release app matrix. |
| Accessibility permission scares users off during onboarding | High | High | Explain before prompting (§7.2 step 4), state plainly that Kivo reads nothing from the screen, and make it verifiable in the source. |
| Model download size causes abandonment | Medium | Medium | Show size and progress honestly; quantise; consider a small model for first-run with an upgrade prompt. |
| Cleanup removes something the user meant | Medium | High | Every stage individually switchable (CLN-4), Raw mode always available, single-undo insertion (DEL-1). |
| Apple or Microsoft ship equivalent OS dictation | Medium | High | Kivo's defensible ground is cleanup quality, cross-platform consistency, and being inspectable. Not the recognition itself. |
| Sole maintainer becomes the bottleneck | High | Medium | Ruthless non-goals (§4), bots doing triage, and a contribution path built before it is needed. |
| Model licence terms change | Low | High | CC-BY-4.0 is irrevocable for released versions. Pin the model version and mirror the artefact. |

## 13. Open questions

1. **Default hotkey.** Fn is ergonomically ideal on macOS and contested with the OS
   dictation shortcut. Right Cmd is safe but less discoverable. *Owner: design. Needed by M5.*
2. **Ship a model, or download on first run?** Bundling adds ~300 MB to every release
   and every update; downloading adds a failure point to onboarding. *Owner: eng. Needed by M6.*
3. **Is Tier 2 LLM polish a v1 feature at all?** It is the most impressive demo and the
   biggest latency and maintenance risk. *Owner: product. Needed by M4.*
4. **How much history by default?** 100 entries is a guess. It is also a privacy
   surface. *Owner: product. Needed by M5.*
5. **Windows ARM64 priority.** Real but small market; doubles the Windows test matrix.
   *Owner: eng. Needed by M6.*

## 14. Assumptions

Stated explicitly so they can be challenged rather than inherited:

- Users are willing to grant accessibility/input permission to a tool they can read the source of.
- A one-time model download of a few hundred megabytes is acceptable to the target user.
- English-first is acceptable for v1, with multilingual following via a model swap.
- Push-to-talk, not always-listening, is the right default — Kivo is a key, not an assistant.
- macOS and Windows desktop is where the unmet need is; mobile is served adequately by the OS.

---

## 15. Related documents

- [README.md](README.md) — what Kivo is, and how to build it
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — system design and the diagram
- [brand/BRAND.md](brand/BRAND.md) — the brand and design system
- [CONTRIBUTING.md](CONTRIBUTING.md) — how to contribute
- [SECURITY.md](SECURITY.md) — reporting vulnerabilities
- [NOTICE](NOTICE) — licence and trademark boundary

---

<sub>Kivo — Copyright © 2026 Gokulakrishnan. Code licensed under the MIT Licence.
The Kivo name and logo are not covered by the code licence; see <a href="NOTICE">NOTICE</a>.</sub>

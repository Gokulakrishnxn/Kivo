# Kivo — Architecture

How speech becomes clean text at your cursor, and why each boundary is where it is.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="diagrams/architecture-dark.svg">
    <img alt="Kivo system architecture: a five-stage on-device pipeline — capture, segment, recognise, clean, deliver — enclosed in a trust boundary showing that no audio leaves the device." src="diagrams/architecture-light.svg" width="100%">
  </picture>
</p>

---

## The shape of the thing

Kivo is a pipeline with a hotkey at one end and a cursor at the other. Everything
between those two points runs in a single Rust process on the user's machine.

```
hotkey ─▶ capture ─▶ ring buffer ─▶ resample ─▶ VAD ─▶ Parakeet ─▶ cleanup ─▶ inject
                                                                       │
                                                                       └─▶ history
```

Two design commitments shape all of it:

1. **The pipeline is synchronous from the user's point of view.** You release the key,
   you get text. There is no queue, no background job, no notification. Every stage
   therefore has a latency budget it must not exceed (see [PRD §9.5](../PRD.md)).
2. **The trust boundary is the process, not the network.** Audio is never written to
   disk and never crosses a socket. This is enforced structurally — the audio buffer
   type is never handed to any module that owns an I/O handle.

---

## Stage 1 — Capture

**Owns:** the global hotkey, the microphone, and the raw sample buffer.

The hotkey listener runs on its own thread with an OS-level hook (`CGEventTap` on
macOS, a low-level keyboard hook on Windows). It must never block: a slow handler here
does not delay Kivo, it delays *the user's entire keyboard*. The handler does nothing
but push an event onto a channel.

Audio capture opens the device on key-down and closes it on key-up. This is deliberate
and slightly more expensive than holding the device open, because it means the OS
microphone indicator tells the truth. A dictation tool that keeps the mic light on
while idle has already lost the argument about privacy, whatever its source code says.

Samples land in a lock-free ring buffer sized for 30 seconds at 16 kHz mono. Anything
longer than 30 seconds is not dictation, it is recording, which is a non-goal. The
buffer is zeroed after each recognition — not merely dropped, but overwritten.

## Stage 2 — Segment

**Owns:** turning device-native audio into something the model expects.

Capture hardware gives whatever it feels like: 44.1 kHz, 48 kHz, stereo, various
formats. `rubato` resamples to the 16 kHz mono f32 Parakeet was trained on. Getting
this wrong degrades accuracy in ways that look like a bad model, so it is tested
against fixed fixtures rather than by ear.

Silero VAD then marks speech and silence. Its job is not to decide when you finished
talking — the hotkey already did that — but to **trim**. Users habitually press the key
half a second before speaking and release it half a second after. Cutting that dead
air is close to free and takes measurable time off inference, because it removes frames
the expensive model would otherwise process.

A short pad is kept on each side. Trimming aggressively clips plosives and costs
accuracy on the first word, which is exactly the word users notice.

## Stage 3 — Recognise

**Owns:** audio in, tokens with timings out. Nothing else.

This stage runs **NVIDIA Parakeet TDT 0.6B**, a FastConformer encoder with a
Token-and-Duration Transducer decoder, exported to ONNX and executed through ONNX
Runtime. The TDT decoder predicts how long each token lasts and skips frames
accordingly, rather than stepping through every frame — which is the property that
makes the latency budget achievable rather than aspirational.

Two things the model gives us that we would otherwise have to build:

- **Punctuation and capitalisation**, natively. No second model in the path.
- **Word-level timestamps**, which Stage 4 needs to detect stutters and
  self-corrections by *timing* rather than by string matching alone.

Execution provider selection is automatic, in order: CoreML on Apple Silicon, CUDA on
NVIDIA hardware, DirectML on other Windows GPUs, CPU otherwise. The CPU path is not a
courtesy — it is a supported configuration with the same latency budget, because a
meaningful share of Windows laptops have nothing better.

**This stage sits behind a trait.** Everything above and below it knows only
`Recogniser: audio → Vec<Word>`. If the M1 spike shows Parakeet missing the CPU budget,
swapping in a distilled Whisper is a contained change rather than a rewrite. Designing
this boundary before choosing the model was intentional.

## Stage 4 — Clean

**Owns:** the difference between what you said and what you meant.

This is the product. Recognition is a solved-enough problem that Kivo's value lives
almost entirely here. It runs in three tiers so the default path never pays for the
expensive one:

| Tier | Does | Cost | Default |
|---|---|---|---|
| **0 — Normalise** | Whitespace, quotes, dashes, spacing around punctuation | microseconds | on |
| **1 — Rules** | Fillers, stutter collapse, user replacements, mode casing | < 1 ms | on |
| **2 — LLM polish** | Self-corrections, sentence repair, register | 200–800 ms | **off** |

Tiers 0 and 1 are deterministic and unit-tested against a fixture corpus. That matters
for more than correctness: a user who does not trust the cleanup will live in Raw mode,
and Raw-mode usage is one of the product signals we watch.

Tier 2 loads a small instruct model through `llama.cpp` only when enabled. Keeping it
out-of-band means the honest answer to "how fast is Kivo?" does not depend on which
settings happen to be on.

## Stage 5 — Deliver

**Owns:** getting text into an application Kivo does not control.

The preferred path sets text directly through the accessibility layer —
`AXUIElement` on macOS, UI Automation's `TextPattern` on Windows. When that works it
is instant, atomic, and undoes in a single keystroke.

It often does not work. Electron apps, terminals, Java applications and games each
mishandle synthetic input in their own way. So Kivo falls back to writing the
clipboard and synthesising a paste, then **restoring the previous clipboard contents**.

That fallback is not a degraded mode. It is the reliability guarantee, and it is
tested every release against a fixed matrix of target applications on both platforms.
Insertion is the largest source of platform-specific bugs in tools of this kind, and
pretending otherwise is how they ship broken.

History, if enabled, is written here — text only, never audio — to a local SQLite
database, encrypted at rest with a key held in the OS keychain. It can be disabled
entirely, in which case nothing touches the disk at all.

---

## The application shell

Tauri v2 wraps the pipeline. The Rust side owns everything above; the webview owns
only settings, onboarding and the recording overlay. This split is why Kivo can idle
under 150 MB while an Electron equivalent starts north of 150 MB before doing anything.

The overlay is deliberately the dumbest window in the system: it never takes focus,
never accepts input, and renders a waveform driven by an input-level event. It exists
so that a dead microphone is visible in the first half-second rather than after you
have finished a paragraph.

## Network

Kivo makes exactly two kinds of outbound request, both listed in Settings with
timestamps:

- **Model download** — once, on first run, to a pinned URL, verified against a pinned
  SHA-256 before the artefact is loaded.
- **Update check** — periodic, opt-out.

There is no audio upload, no transcript upload, no telemetry, no crash reporting, and
no account. Settings shows a live network panel whose normal state is empty, so the
claim is checkable without reading any code — and the code is there for those who want
to check properly.

## Threading

| Thread | Responsibility | Constraint |
|---|---|---|
| Hotkey hook | OS keyboard events | Must never block. Pushes to a channel and returns. |
| Audio callback | Device samples → ring buffer | Real-time. No allocation, no locks, no logging. |
| Pipeline worker | Segment → recognise → clean → deliver | Owns the latency budget end to end. |
| UI (webview) | Settings, onboarding, overlay | Never touches audio buffers. |

The audio callback and the hotkey hook are the two places where ordinary Rust
conveniences are banned. Both run in contexts where being slow is not a performance
problem but a correctness one.

---

## Repository layout

The application scaffold arrives in Milestone 2; this is the shape it lands in.

```
Kivo/
├── src/                    interface — React + TypeScript
│   ├── onboarding/
│   ├── settings/
│   └── overlay/
├── src-tauri/              core — Rust
│   └── src/
│       ├── hotkey/         global hotkey, per platform
│       ├── audio/          capture, ring buffer, resample, VAD
│       ├── asr/            Recogniser trait + Parakeet implementation
│       ├── clean/          tiers 0, 1, 2
│       ├── inject/         text delivery, per platform
│       ├── history/        SQLite, encryption
│       └── models/         download, SHA-256 verification, cache
├── brand/                  logo, tokens, guidelines
├── web/                    the website
└── docs/                   this document and the diagrams
```

---

## Related

- [PRD.md](../PRD.md) — requirements, budgets, milestones and open questions
- [BRAND.md](../brand/BRAND.md) — the design system this interface is built from
- [CONTRIBUTING.md](../CONTRIBUTING.md) — how to work on any of the above

<sub>The diagrams in `docs/diagrams/` are the canonical version of the picture above.
Update them alongside any change to these boundaries.</sub>

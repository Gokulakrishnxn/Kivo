# Kivo — Brand Kit

The complete visual system for Kivo: what the marks are, how the system is built, and
where the boundaries sit. Everything here is the canonical source. If a product screen
and this document disagree, this document is right.

**Interactive version:** [the brand kit page](../web/brand.html) renders every token,
mark and specimen in both themes.

---

## 1. The idea

Kivo turns voice into clean text. The brand has one job: make that transformation feel
**precise** rather than magical.

That single decision explains everything downstream. Voice products default to purple
gradients, glowing orbs and soft blur — the visual language of *something happening
that you cannot see*. Kivo goes the other way. It is monochrome, geometric, hairline-ruled
and flat, because it is a tool that does a specific thing accurately, and it wants to
be trusted rather than admired.

**Personality:** precise, quiet, unhurried, technical without being cold.
**Not:** playful, futuristic, chatty, assistant-like, "AI".

---

## 2. The mark

Kivo's mark is the product drawn literally: **three waveform bars on the left resolving
into three lines of set text on the right.** Speech, then the gap where the work
happens, then text.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="logo/kivo-lockup-white.svg">
    <img src="logo/kivo-lockup-black.svg" alt="The Kivo lockup: the waveform-to-text mark beside the Kivo wordmark" width="320">
  </picture>
</p>

### Construction

Every element is drawn on a **32-unit grid** from a single **3.2-unit stroke** with
fully rounded caps (`rx` = 1.6, exactly half the stroke). There is no second weight and
no second radius. The vertical bars and the horizontal lines are the same stroke turned
ninety degrees — which is the whole idea of the product expressed as geometry.

The gap between the two groups is deliberately wide. It is the only empty space in the
mark, and it is where the transformation is implied. Do not close it.

### Variants

| File | Use |
|---|---|
| [`kivo-mark.svg`](logo/kivo-mark.svg) | The mark, inheriting `currentColor`. Preferred for web and app code. |
| [`kivo-mark-black.svg`](logo/kivo-mark-black.svg) · [`kivo-mark-white.svg`](logo/kivo-mark-white.svg) | Fixed-colour, for contexts that cannot inherit. |
| [`kivo-mark-compact.svg`](logo/kivo-mark-compact.svg) | **Two bars, two lines, heavier stroke.** For 16–23 px. |
| [`kivo-logotype.svg`](logo/kivo-logotype.svg) | The word alone. |
| [`kivo-lockup.svg`](logo/kivo-lockup.svg) | Mark + word. The default logo. |
| [`kivo-app-icon.svg`](logo/kivo-app-icon.svg) | Black squircle, white glyph. Dock, tray, installer. |
| [`kivo-app-icon-inverse.svg`](logo/kivo-app-icon-inverse.svg) | White squircle with a hairline, for dark tiles. |
| [`favicon.svg`](logo/favicon.svg) | Theme-aware tile. Inverts with the browser. |

### Minimum sizes

These are not suggestions. The full mark's three text lines merge into a smudge below
24 px, which is why the compact variant exists.

| Asset | Minimum | Below that |
|---|---|---|
| Full mark | 24 px | use the compact mark |
| Compact mark | 16 px | use nothing; use the word |
| Lockup | 120 px wide | use the mark alone |
| Logotype | 64 px wide | — |
| App icon | 32 px | — |

### Clear space

Reserve clear space equal to **the height of one waveform bar in the mark** (¼ of the
mark's height) on all four sides. Nothing enters it — no text, no rules, no edges.

### Misuse

Do not: recolour it, add a gradient, add a shadow, outline it, rotate it, stretch it
non-uniformly, reorder the elements, close the central gap, place it on a busy image,
put it inside another shape, or rebuild the wordmark in a different typeface. The
logotype is drawn, not set — there is no font that produces it.

---

## 3. Colour

**Kivo is monochrome. There is no accent colour, and adding one is a brand change, not
a design decision.**

Emphasis is created by **inversion** and **weight** instead of hue. A primary button is
not "the blue one", it is the one filled with ink. This is a constraint that pays for
itself: it forces hierarchy to be carried by structure, and it makes the product legible
to everyone regardless of colour vision.

### The ink ramp

| Token | Hex | Typical use |
|---|---|---|
| `--kivo-paper` | `#FFFFFF` | Page and surface, light theme |
| `--kivo-ink-50` | `#F7F7F7` | Subtle fills, alternating rows |
| `--kivo-ink-100` | `#EFEFEF` | Inset wells, code blocks |
| `--kivo-ink-200` | `#E2E2E2` | **Default hairline border** |
| `--kivo-ink-300` | `#CBCBCB` | Strong border, dividers under emphasis |
| `--kivo-ink-400` | `#A6A6A6` | Disabled text, dark-theme secondary text |
| `--kivo-ink-500` | `#7C7C7C` | Muted text, captions |
| `--kivo-ink-600` | `#5C5C5C` | **Secondary text** |
| `--kivo-ink-700` | `#404040` | Dark-theme strong borders |
| `--kivo-ink-800` | `#292929` | Dark-theme borders, raised surfaces |
| `--kivo-ink-900` | `#171717` | Dark-theme surface |
| `--kivo-ink-950` | `#0A0A0A` | **Primary text, primary action** |

Ink is `#0A0A0A`, not `#000000`. Pure black against pure white is harsh at paragraph
length and reads as unconsidered. Pure black is reserved for `--kivo-bg-inset` in the
dark theme, where it is a *recess* below the surface.

### Semantic tokens

Product code must never reference the ramp directly. Use `--kivo-text`,
`--kivo-border`, `--kivo-action` and their siblings, defined in
[`tokens/kivo-tokens.css`](tokens/kivo-tokens.css). This is what makes the dark theme a
token swap rather than an audit.

### Dark theme

Kivo's dark theme is a **true inversion, not a dimming**. Paper becomes ink; ink becomes
paper. Both themes are first-class and both are checked before any interface change ships.

### The one exception

Destructive confirmations may use the platform's native danger colour. It is functional,
never decorative, must never exceed roughly 1 % of the visible surface, and is
explicitly outside the brand palette. Nothing else gets an exception.

### Contrast

All text meets **WCAG 2.2 AA**. Body text on paper (`ink-950` on `paper`) is 19.6:1.
The lightest text permitted on paper is `ink-500` at 4.8:1, and only at 16 px or larger.
`ink-400` is for disabled states and never carries meaning on its own.

---

## 4. Typography

| Role | Typeface | Notes |
|---|---|---|
| Interface, headings, body | **Inter** | Variable. SIL Open Font Licence. Neutral, excellent at small sizes, real tabular figures. |
| Code, data, technical detail | **JetBrains Mono** | Apache 2.0. Used for anything measured, versioned or typed by a machine. |

Both are open-licensed and can ship with the app — a paid brand typeface in an MIT
project is a licence trap for every fork.

Kivo uses mono **as a texture, not just for code**: version strings, timings, key names,
file paths and stage labels are all set in JetBrains Mono. It signals "this is a
measured fact" and it is a large part of why the brand reads as technical without
needing a single icon.

### Scale

| Token | Size | Weight | Tracking | Use |
|---|---|---|---|---|
| `--kivo-text-5xl` | 60 | 600 | −0.035em | Hero display |
| `--kivo-text-4xl` | 48 | 600 | −0.035em | Page title |
| `--kivo-text-3xl` | 38 | 600 | −0.022em | Section heading |
| `--kivo-text-2xl` | 30 | 600 | −0.022em | Sub-section |
| `--kivo-text-xl` | 24 | 600 | −0.022em | Card heading |
| `--kivo-text-lg` | 20 | 400 | −0.011em | Lead paragraph |
| `--kivo-text-md` | 17 | 400 | −0.011em | Long-form body |
| `--kivo-text-base` | 16 | 400 | −0.011em | Body |
| `--kivo-text-sm` | 14 | 400/500 | −0.011em | Interface default |
| `--kivo-text-xs` | 12 | 500 | 0 | Metadata |
| `--kivo-text-2xs` | 11 | 600 | +0.10em | Eyebrows, uppercase labels |

**Tracking tightens as type grows.** Display type set at default tracking looks loose
and amateur; the negative values above are doing real work.

**Line height:** 1.06 display · 1.18 headings · 1.62 body.
**Measure:** 62–72 characters. Never let a paragraph run the full width of a wide screen.
**Weights:** 400, 500, 600 only. No 700, no italics in the interface.

---

## 5. Space, shape and elevation

**Grid:** 4 px base. Every dimension is a multiple of 4.
**Container:** 1120 px, 24 px gutters. Prose narrows to 720 px.

**Radius:** 0 · 4 · 6 · 8 · 12 · 16 · 22 · full. Kivo is squared-off — buttons and
inputs use 6–8 px, cards 12 px, the app icon squircle is 22 % of its edge. Pills are
reserved for status badges only.

**Elevation — the important rule: Kivo separates surfaces with hairlines, not shadows.**
A 1 px `--kivo-border` is the default way to say "this is a distinct thing". Shadows are
permitted only on chrome that genuinely floats above the operating system — the
dictation overlay and menus — and there are exactly two, defined as `--kivo-shadow-hud`
and `--kivo-shadow-menu`. Nothing else casts a shadow. Ever.

---

## 6. Motion

Dictation happens in under 300 milliseconds. Animation that draws attention to itself
makes a fast product feel slow.

| Token | Duration | Use |
|---|---|---|
| `--kivo-duration-instant` | 90 ms | Hover, focus, press |
| `--kivo-duration-fast` | 140 ms | Toggles, small reveals |
| `--kivo-duration-normal` | 220 ms | Panels, overlay entry |
| `--kivo-duration-slow` | 380 ms | Full-screen transitions only |

Easing: `--kivo-ease-standard` `cubic-bezier(0.2, 0, 0, 1)` for almost everything —
it leaves immediately and settles gently. Never `linear`, except for the live waveform,
which is data and must not be smoothed.

**The recording overlay** is the one place motion is expressive: the waveform is driven
by real input level, at real time, with no interpolation. A user must be able to tell a
dead microphone from a quiet room instantly.

`prefers-reduced-motion: reduce` sets every duration to zero. This is honoured in the
tokens themselves, so it cannot be forgotten in a component.

---

## 7. Voice and tone

Write like a precise colleague, not like a product.

- **Plain and specific.** "Text appears at your cursor in under 300 ms", not "lightning-fast, seamless dictation".
- **Second person, active voice.** "You hold a key. Kivo types."
- **Numbers over adjectives.** Claims should be checkable.
- **Never anthropomorphise.** Kivo does not listen, understand, think or help. It records, recognises and inserts. It is a tool, and the brand's credibility rests on not pretending otherwise.
- **No exclamation marks. No emoji in product copy.** (README and issue templates may use them sparingly for navigation.)
- **Own failure plainly.** "Kivo could not insert text into this app, so it copied it to your clipboard instead." Not "Oops! Something went wrong."

Capitalisation: **Kivo** always, never KIVO or kivo. Sentence case for all headings and
buttons, never Title Case.

One-liners, in order of preference:
> Turns voice into clean text.
> Faster than typing. Entirely on your device.
> Hold a key. Speak. It's text.

---

## 8. Using the brand

The code in this repository is MIT licensed. **The Kivo name and the assets in this
directory are not covered by that licence.** See [NOTICE](../NOTICE) for the full terms.

**You may** refer to Kivo by name, reproduce the logo unmodified to write or talk about
the project, and say your work is a fork of Kivo.

**You may not** use the Kivo name or logo as the identity of your own fork, product or
company, modify the logo and still present it as Kivo, or imply endorsement.

If you fork Kivo, please give it your own name and your own mark. The system in this
document is a good starting point for building one.

---

## 9. Files

```
brand/
├── BRAND.md                    this document
├── logo/
│   ├── kivo-mark.svg           mark · currentColor
│   ├── kivo-mark-black.svg
│   ├── kivo-mark-white.svg
│   ├── kivo-mark-compact.svg   16–23 px variant
│   ├── kivo-mark-compact-black.svg
│   ├── kivo-mark-compact-white.svg
│   ├── kivo-logotype.svg       drawn wordmark
│   ├── kivo-logotype-black.svg
│   ├── kivo-logotype-white.svg
│   ├── kivo-lockup.svg         mark + wordmark · the default logo
│   ├── kivo-lockup-black.svg
│   ├── kivo-lockup-white.svg
│   ├── kivo-app-icon.svg
│   ├── kivo-app-icon-inverse.svg
│   └── favicon.svg
└── tokens/
    ├── kivo-tokens.css         the implementation source of truth
    └── kivo-tokens.json        W3C design-token format, for tooling
```

Every asset is SVG. There are no raster masters — the marks are geometric and any PNG
you need should be exported from these files at the size you need it.

---

<sub>Kivo — Copyright © 2026 Gokulakrishnan. Code MIT licensed; brand assets are not.
See <a href="../NOTICE">NOTICE</a>.</sub>

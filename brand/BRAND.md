# Kivo — Brand Kit

The complete visual system for Kivo. Everything here is canonical: if a product screen
and this document disagree, this document is right.

**Interactive version:** [the brand kit page](../web/brand.html) renders every token,
mark, character and specimen live.

> **Version 2.0 — August 2026.** Kivo previously used a strictly monochrome system.
> It now uses the cream-parchment system described below. The change was deliberate and
> total; nothing from v1 carries forward except the geometry of the mark itself.

---

## 1. The idea

**A storybook spread on cream parchment.**

Kivo's chrome is deliberately quiet — warm paper, hairline borders, no shadows — so a
small cast of hand-drawn characters and a scatter of punctuation can carry all the
personality. The interface recedes; the illustration does the emotional work.

That split is the whole system, and it is also the honest shape of the product. Kivo is
a small friendly thing that sits in your menu bar, and it is also a piece of software
holding a microphone permission. The look is warm. The claims are exact.

**Personality:** warm, playful, unhurried, precise where it counts.
**Not:** corporate, futuristic, glossy, gradient-y, or breathlessly "AI".

### The one rule that governs everything else

> **The illustrations are playful. The claims are not.**

A storybook look does not license a storybook promise. Kivo records, recognises and
inserts. It does not listen, understand, think or help — and no amount of charm in the
drawing changes what the copy is allowed to say. See §7.

---

## 2. The mark

**The cursor K.** A vertical cursor and two angled strokes form a K. It connects the
product name to the place dictation lands: the text cursor. Rounded ends keep the
geometry compatible with Kivo’s warm illustrated identity.

Updated at the owner’s request on 16 October 2026. This replaces the multicolour
waveform mark. The existing drawn wordmark stays unchanged.

### Construction and colour

A 32-unit grid. The stem is 5 units wide, from (5, 5) to (10, 27), with 2.5-unit
corner radii. The arms follow (25, 7), (15, 16), (25, 25), with a 5-unit stroke and
round caps and joins. Preserve the open gap between the stem and arms.

Use ink `#121212` on light backgrounds and white on ink. The entire mark reverses
together. No gradients, shadows, multicolour fills, rotation or distortion.

### Files and sizes

- `logo/kivo-mark.svg`: adaptable `currentColor` mark.
- `logo/kivo-mark-colour.svg`: fixed ink mark; legacy filename retained for compatibility.
- `logo/kivo-mark-compact.svg`: the same silhouette for small placements.
- `logo/kivo-lockup.svg`: adaptable mark and unchanged drawn wordmark.
- `logo/kivo-lockup-colour.svg`: fixed ink lockup; legacy filename retained.
- `logo/kivo-logotype.svg`: unchanged wordmark.
- `logo/kivo-app-icon.svg`: cursor K on a paper squircle.
- `logo/favicon.svg`: cursor K on paper.

Use the mark at 16 px or larger and the lockup at 120 px or larger. Leave at least
5 grid units of clear space around the mark. Never recreate the wordmark with a font.

---

## 3. The cast

Five characters and a scatter of punctuation. They are the reason the interface can stay
as quiet as it does.

| Character | Colour | Is |
|---|---|---|
| **mic** | Sun `#FFCD6C` | the listening one |
| **bubble** | Sky `#64C6FF` | what you said |
| **page** | Green `#00C978` | the clean text it hands back |
| **caret** | Ember `#FF3E00` | where the text lands |
| **wave** | Pink `#FF58AE` | your voice, mid-air |

**Drawing rules.** Flat fills from the accent palette. Strokes are **`#343433` at 3 units**,
never black, never heavier. Dot eyes and stick limbs with a dot at each end. Organic
radii of 40–72 px. No gradients, no shading, no outlines-within-outlines.

**The confetti is punctuation** — full stop, comma, quotation mark, question mark,
exclamation, em dash — plus a star, sparkle, keycap and heart. This is not decoration
chosen at random: punctuation is precisely what Kivo adds to your speech.

**Placement.** Scatter asymmetrically into negative space. Characters flank centred type
and **never overlap it**. Confetti never sits inside a card's text column. Two clusters
balancing a centred text block is the hero pattern; single small accents are the pattern
everywhere else.

---

## 4. Colour

Warm paper, one dark action, bright markers. **Ink is the only fill a button ever gets.**
Every other colour is a marker on text or a fill in an illustration.

### Surfaces

| Token | Hex | Role |
|---|---|---|
| `--kivo-canvas` | `#FBFAF9` | Page background and nav. Warm paper, not screen white. |
| `--kivo-stone` | `#F2F0ED` | **The inset border tone.** Also secondary panels. |
| `--kivo-sand` | `#F6F4EF` | Secondary pill button, light tag surfaces. |
| `--kivo-white` | `#FFFFFF` | Feature cards — only here does the hairline register. |
| `--kivo-ink` | `#121212` | Primary action fill. The one dark surface. |
| `--kivo-black` | `#000000` | The dark feature card and its shadow base. |

### Text

| Token | Hex | Role |
|---|---|---|
| `--kivo-heading` | `#343433` | Headings, nav, **and every illustration stroke**. |
| `--kivo-body` | `#474645` | Body copy. Warm brown, never cool grey. |
| `--kivo-muted` | `#7E7E7D` | Helper text, inactive nav, tertiary labels. |

### Accents

| Token | Hex | Role |
|---|---|---|
| `--kivo-stone-border` | `#E5D5C3` | Warm hairline on illustrations and the app icon. |
| `--kivo-blue` | `#0086FC` | Inline links and list emphasis. **Never a button fill.** |
| `--kivo-sky` | `#64C6FF` | Illustration fill. |
| `--kivo-alt-blue` | `#00B2FF` | Secondary illustration fill, icon accents. |
| `--kivo-green` | `#00C978` | Accent text, tags, short emphasised phrases. |
| `--kivo-mint` | `#00CA48` | Wash behind content. Positive status pills. |
| `--kivo-ember` | `#FF3E00` | **The signature link colour.** |
| `--kivo-sun` | `#FFCD6C` | Illustration fill. |
| `--kivo-gold` | `#D48F00` | Yellow accent text, where honey is too light to read. |
| `--kivo-honey` | `#FFBB26` | Wash behind content. Pending status pills. |
| `--kivo-pink` | `#FF58AE` | Badge fill and illustration accent. |
| `--kivo-violet` | `#9F4FFF` | Wash and icon accent. **Never the primary CTA.** |
| `--kivo-red` | `#FF2B3A` | Destructive and error only. Nothing else. |

### The rules that matter

1. **Ink `#121212` is the only button fill.** Not blue, not ember, not violet.
2. **Ember `#FF3E00` is the demo-link colour** — underlined text, never a filled shape.
3. **Blue is for inline links and list emphasis**, never a background.
4. **Red is destructive only.** The warm-accent slot belongs to ember.
5. **No gradients, anywhere.** The system is strictly flat.
6. **Never separate surfaces white-on-white.** Shift to stone, or to ink.

### Accent text vs accent fill

**The bright accents are fills, not text.** Every one of them fails WCAG AA as small
text on canvas — green is 2.09:1, gold 2.61:1, pink 2.78:1, ember 3.39:1, blue 3.46:1.
That is a property of the palette, not a mistake to design around: those hues exist for
illustration fills, washes and badge grounds.

When an accent has to carry **words**, use the matching `-ink` tint. Each is the same
hue darkened to the lightest value that clears 4.5:1 on canvas.

| Fill | As text | Tint token | Ratio |
|---|---|---|---|
| `--kivo-green` `#00C978` | 2.09:1 ✗ | `--kivo-green-ink` `#00834E` | 4.62:1 ✓ |
| `--kivo-blue` `#0086FC` | 3.46:1 ✗ | `--kivo-blue-ink` `#0072D6` | 4.60:1 ✓ |
| `--kivo-ember` `#FF3E00` | 3.39:1 ✗ | `--kivo-ember-ink` `#D63400` | 4.63:1 ✓ |
| `--kivo-gold` `#D48F00` | 2.61:1 ✗ | `--kivo-gold-ink` `#9B6800` | 4.61:1 ✓ |
| `--kivo-pink` `#FF58AE` | 2.78:1 ✗ | `--kivo-pink-ink` `#DF0073` | 4.59:1 ✓ |
| `--kivo-violet` `#9F4FFF` | 4.00:1 ✗ | `--kivo-violet-ink` `#943BFF` | 4.65:1 ✓ |

Badge labels use `--kivo-badge-ink` `#0A2E14`, which clears 4.5:1 on every accent fill
(mint 6.7:1, honey 8.8:1, sky 7.8:1).

`--kivo-muted` is `#6D6D6C`, not the `#7E7E7D` the look was derived from — that value is
3.90:1 on canvas and fails at the 12–15 px sizes helper text is actually set at.

### Contrast floor

**All text meets WCAG 2.2 AA.** Body on canvas is 11.4:1. Nothing ships below 4.5:1, and
that is verified by an automated sweep across thirteen viewport widths rather than by eye.

### Theme

Kivo's marketing surfaces are **light only**. The cream canvas is the brand; there is no
dark variant of it. The *application* follows the operating system's theme, using ink
`#121212` as its dark ground — that is a product concern, not a brand one.

---

## 5. Typography

| Role | Typeface | Notes |
|---|---|---|
| Display, hero headings | **Bricolage Grotesque**, weight 500 | Slightly condensed, slightly playful. Makes a heading feel drawn rather than set. |
| Everything else | **Inter**, 400 / 500 / 600 | The utility layer. |
| Measured facts | **JetBrains Mono** | Versions, timings, hashes, key names. |

All three are open-licensed, so a fork inherits no licence trap.

**Never set display sizes in Inter.** The display face is what stops the page reading as
a generic SaaS template.

### Scale

| Role | Size | Line height | Tracking |
|---|---|---|---|
| display | 68 px | 75 | −0.031em |
| heading-lg | 44 px | 53 | −0.020em |
| heading | 23 px | 25 | −0.019em |
| subheading | 19 px | 27 | −0.016em |
| body | 17 px | 26 | −0.013em |
| caption | 15 px | 22 | −0.014em |
| micro | 12 px | 19 | −0.010em |

Tracking tightens as type grows; display set at default tracking looks loose and amateur.
Use `text-wrap: balance` on display and `heading-lg` — an orphaned last word ruins a hero.

---

## 6. Space, shape and elevation

**Grid:** 4 px base, comfortable density.
**Container:** 1200 px. Prose narrows to 680 px.
**Section gap:** 80–120 px. **Card padding:** 32 px. **Element gap:** 8–12 px.

**Radius:** 2 small · 6 badge · **10 cards and nav (the default)** · 32 buttons ·
40 icons · 72 illustration · 9999 pills.

### Elevation — the signature technique

> **Cards are defined by a 1 px _inset_ hairline in stone, not by a drop shadow.**
> `box-shadow: inset 0 0 0 1px #F2F0ED`

This is what gives every page its flat, pressed-into-paper quality. Nothing floats.

The only exceptions: the dark feature card may use `0 0 24px rgba(0,0,0,.15)`, and a
card may use a whisper-light lift at `rgba(0,0,0,.04)`. **Never heavier than that.**

## 6b. Motion

Fast 140 ms, normal 220 ms, easing `cubic-bezier(0.2, 0, 0, 1)`. Mascots may bob gently
(±7 px, ~4.5 s) — that is the only decorative animation permitted, and
`prefers-reduced-motion: reduce` stops it along with everything else.

---

## 7. Voice and tone

Warm on the surface, exact underneath.

- **Plain and specific.** "Text appears at your cursor in under 300 ms", not "lightning-fast dictation".
- **Second person, active voice.** "You hold a key. Kivo types."
- **Numbers over adjectives.** Claims should be checkable.
- **Never anthropomorphise Kivo.** It records, recognises and inserts. It does not listen,
  understand, think or help. **The cast is allowed a face; the product is not.**
- **No exclamation marks. No emoji in product copy.**
- **Own failure plainly.** "Kivo could not insert text into this app, so it copied it to
  your clipboard instead." Not "Oops! Something went wrong."

Capitalisation: **Kivo** always. Sentence case for all headings and buttons.

One-liners, in order of preference:
> Turns voice into clean text.
> Faster than typing. Entirely on your device.
> Hold a key. Speak. It's text.

---

## 8. Using the brand

The code in this repository is MIT licensed. **The Kivo name, the marks and the cast in
this directory are not covered by that licence.** See [NOTICE](../NOTICE).

**You may** refer to Kivo by name, reproduce the logo unmodified to write about the
project, and say your work is a fork of Kivo.

**You may not** use the Kivo name, logo or characters as the identity of your own fork,
product or company, modify them and still present them as Kivo, or imply endorsement.

If you fork Kivo, give it your own name and your own mark. This document is a reasonable
template for building one.

---

## 9. Files

```
brand/
├── BRAND.md                      this document
├── logo/
│   ├── kivo-lockup-colour.svg    THE default logo
│   ├── kivo-mark-colour.svg      the mark, fixed ink
│   ├── kivo-mark.svg             single-colour · currentColor
│   ├── kivo-mark-compact.svg     small-size mark
│   ├── kivo-logotype.svg         the drawn word
│   ├── kivo-app-icon.svg         paper squircle
│   └── favicon.svg
├── illustration/
│   ├── mascot-mic.svg            the listening one
│   ├── mascot-bubble.svg         what you said
│   ├── mascot-page.svg           the clean text
│   ├── mascot-caret.svg          where it lands
│   ├── mascot-wave.svg           your voice, mid-air
│   └── confetti-*.svg            punctuation ×10
└── tokens/
    ├── kivo-tokens.css           the implementation source of truth
    └── kivo-tokens.json          W3C design-token format, for tooling
```

Every asset is SVG. There are no raster masters — export a PNG from these at whatever
size you need it.

---

<sub>Kivo — Copyright © 2026 Gokulakrishnan. Code MIT licensed; brand assets are not.
See <a href="../NOTICE">NOTICE</a>.</sub>

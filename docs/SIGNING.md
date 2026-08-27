# Signing and notarising Kivo for macOS

Everything on this page that touches your Apple ID password is a step **you**
run. Nothing in this repository asks for, reads, transmits or stores those
credentials, and no maintainer or agent should ever be handed them.

---

## What you get without any of this

`./scripts/sign-macos.sh` with no arguments produces an **ad-hoc signed** app.
No certificate, no Apple account, no annual fee. It is valid, it runs on the Mac
that built it, and `codesign --verify --deep --strict` passes.

It will **not** open on anyone else's Mac without them explicitly overriding
Gatekeeper. On **macOS 15 (Sequoia) and later** that means System Settings →
Privacy & Security → *Open Anyway* — Apple removed the old Control-click → Open
bypass, so any instructions telling users to Control-click are out of date.
Gatekeeper rejects an ad-hoc build by design:

```
src-tauri/target/release/bundle/macos/Kivo.app: rejected
```

That is the correct outcome for an unsigned build, not a failure.

## What you need to go further

| | |
|---|---|
| **Apple Developer Program** | $99/year. A free Apple ID is not enough — free accounts cannot issue a Developer ID. |
| **Developer ID Application certificate** | For distributing outside the Mac App Store. |
| **An app-specific password** *or* an App Store Connect API key | For notarisation. |

**Kivo can never go in the Mac App Store.** Its entire job is reading a global
hotkey and injecting text into other applications, which is precisely what the
App Sandbox exists to prevent. Developer ID is the only distribution route, and
`Entitlements.plist` reflects that.

---

## 1 · Get the certificate

1. Join the [Apple Developer Program](https://developer.apple.com/programs/).
2. In [Certificates, Identifiers & Profiles](https://developer.apple.com/account/resources/certificates/list),
   create a **Developer ID Application** certificate.
3. Download and double-click it to install into your login keychain.

Confirm it landed:

```bash
security find-identity -v -p codesigning
```

You want a line like `Developer ID Application: Your Name (ABCDE12345)`. The
parenthesised value is your **Team ID**.

## 2 · Store notarisation credentials — you, not anyone else

Create an app-specific password at [appleid.apple.com](https://appleid.apple.com)
→ Sign-In and Security → App-Specific Passwords. Then hand it to `notarytool`,
which stores it in **your** keychain:

```bash
xcrun notarytool store-credentials "kivo" \
  --apple-id "you@example.com" \
  --team-id "ABCDE12345"
```

It prompts for the app-specific password interactively. After this, nothing else
ever needs to see it — the profile name `kivo` is all the build refers to.

## 3 · Build, sign, notarise

```bash
pnpm tauri build --bundles app,dmg

APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (ABCDE12345)" \
NOTARY_PROFILE=kivo \
./scripts/sign-macos.sh
```

The script signs the nested dylibs first (innermost outward, or the outer
signature seals a stale hash), applies the hardened runtime and entitlements,
uploads to Apple, waits, staples the ticket, and re-checks with `spctl`.

A clean finish looks like:

```
src-tauri/target/release/bundle/macos/Kivo.app: accepted
source=Notarized Developer ID
```

## 4 · Releasing from CI instead

[`.github/workflows/release.yml`](../.github/workflows/release.yml) does the same
thing on a tag push. It needs six repository secrets, which **you** add under
Settings → Secrets and variables → Actions:

| Secret | What it is |
|---|---|
| `APPLE_CERTIFICATE` | Your `.p12`, base64-encoded |
| `APPLE_CERTIFICATE_PASSWORD` | The password you set when exporting the `.p12` |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Your Name (ABCDE12345)` |
| `APPLE_ID` | Your Apple ID email |
| `APPLE_PASSWORD` | The app-specific password |
| `APPLE_TEAM_ID` | `ABCDE12345` |

To export the `.p12`: Keychain Access → your Developer ID certificate →
right-click → Export → `.p12`, then

```bash
base64 -i certificate.p12 | pbcopy
```

and paste that as `APPLE_CERTIFICATE`.

**These are credentials.** Add them through GitHub's own secrets UI. Do not paste
them into an issue, a pull request, a chat window, or a file in this repository.

---

## Why permissions keep resetting while you develop

macOS ties Accessibility and Microphone grants to an app's code signature. Every
rebuild changes it, so macOS treats the new build as a different app and the
grant you gave five minutes ago no longer applies — the entry sits in System
Settings looking enabled while doing nothing.

Ad-hoc signing does not fix this; the hash still changes each build. The reliable
workaround while developing:

```bash
# Remove the stale grant, then re-add the rebuilt app.
tccutil reset Accessibility sh.kivo.desktop
```

Then toggle Kivo off and on in System Settings → Privacy & Security →
Accessibility. With a real Developer ID certificate the signature is stable
across builds and this stops being a problem.

<sub>Kivo — Copyright © 2026 Gokulakrishnan. MIT licensed.</sub>

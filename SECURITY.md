# Security Policy

Kivo captures microphone audio, holds transcribed text, and injects keystrokes into
other applications. That is a serious set of capabilities, and security reports are
treated accordingly.

## Reporting a vulnerability

**Please do not open a public issue.**

Report privately through GitHub:

**→ [Open a private security advisory](https://github.com/Gokulakrishnxn/Kivo/security/advisories/new)**

This is visible only to you and the maintainer, and it gives us a private space to
develop and test a fix before anything becomes public.

If GitHub advisories are not available to you, send a direct message to
[@Gokulakrishnxn](https://github.com/Gokulakrishnxn) asking for a private channel.
Do not include vulnerability details in that first message.

### What to include

- What the vulnerability is, and what an attacker could do with it
- Steps to reproduce, ideally minimal
- Affected version, OS and hardware
- Any proof-of-concept — as text, never as a real audio recording
- Whether you intend to disclose publicly, and on what timeline

### What to expect

| | |
|---|---|
| Acknowledgement | Within **72 hours** |
| Initial assessment | Within **7 days** |
| Fix or mitigation plan | Within **30 days** for high and critical severity |
| Public disclosure | Coordinated with you, after a fix ships |

Kivo is maintained by one person. If you have not heard back within 72 hours, please
follow up rather than assuming the report was received — the failure mode here is a
missed notification, not disinterest.

Reporters are credited in the advisory and the release notes unless you would rather
not be. There is no bug bounty; Kivo is an unfunded open source project, and it would
be dishonest to imply otherwise.

## Scope

**In scope**

- Audio or transcript data leaving the device by any path
- Anything written to disk that the user was told would not be
- Privilege escalation through Kivo's accessibility or input-injection permissions
- Injecting text into an application other than the focused one, or at a time the user
  did not initiate
- Clipboard contents being leaked or not restored after the paste fallback
- Model artefacts being loaded without SHA-256 verification, or verification being bypassable
- Tampering with the update channel, or an unsigned artefact being accepted
- Local privilege escalation via the installer, the updater, or file permissions
- Extraction of the history database or its encryption key by another local user
- Supply-chain issues in Kivo's dependencies or CI

**Out of scope**

- Vulnerabilities in the underlying operating system, unless Kivo makes them exploitable
- Attacks requiring physical access to an unlocked machine already logged in as the user
- Social engineering of the maintainer or contributors
- Missing hardening that has no demonstrable impact
- Reports from automated scanners with no working proof of concept
- Denial of service against the user's own machine by the user
- The absence of a feature (for example, "Kivo does not sandbox X") without a concrete
  attack that follows from it

## Kivo's security properties

These are the claims Kivo makes. **A demonstrated violation of any of them is a
vulnerability**, and should be reported as one even if it does not look like a classic
security bug:

1. Audio never leaves the device, and is never written to disk.
2. Transcripts never leave the device.
3. There is no telemetry, no analytics, and no crash reporting.
4. The only outbound requests are the model download and the opt-out update check.
5. Model artefacts are verified against a pinned SHA-256 before being loaded.
6. The microphone is opened only while the hotkey is held, and released immediately after.
7. Text is inserted only into the focused application, and only in response to a
   user-initiated dictation.
8. The clipboard is restored to its previous contents after the paste fallback.
9. History, when enabled, is text only — never audio — and is stored in a SQLite file
   readable solely by the user's own account (mode 0600). Disabling history deletes
   that file, so "off" means gone rather than merely ignored.

> **Not yet true:** history is *not* encrypted at rest. DEL-6 in the PRD calls for a
> key held in the OS keychain and that is not implemented. Anyone who can read your
> user account can read your transcript history. Treat that as the current guarantee
> and no more.

## Known advisories we are not acting on

Kept here so the reasoning is public and nobody re-investigates it.

| Advisory | Package | Why it is not being fixed |
|---|---|---|
| Unsoundness in `glib`'s `Iterator` impls (medium) | `glib` 0.18.5 | Reached only through `gtk` → Tauri's **Linux** backend. `cargo tree --target aarch64-apple-darwin` resolves zero `glib` nodes; the Linux target resolves 29. Kivo ships macOS and Windows only, so the code is never compiled. It also cannot be patched locally — `gtk` 0.18.2 pins `glib ^0.18`, so a bump to 0.20 is rejected by the resolver. Revisit if Linux support lands, or when Tauri updates its GTK stack. |

If you can show that any entry in this table *is* reachable on a shipping build,
that is a valid vulnerability report and we would like to hear it.

## Supported versions

| Version | Supported |
|---|---|
| Latest release | ✅ |
| Previous minor | ✅ security fixes only |
| Anything older | ❌ |

Kivo is pre-1.0. Until v1.0 ships, only the latest release is supported. Builds from
`main` are not supported at all.

## For contributors

If you are writing code that touches audio buffers, the network, the filesystem, the
clipboard, or input injection, re-read the properties above first. A pull request that
breaks one of them will be closed rather than revised — not as a judgement of the work,
but because those nine lines are the reason anyone can use this tool at their job.

<sub>Kivo — Copyright © 2026 Gokulakrishnan. MIT licensed.</sub>

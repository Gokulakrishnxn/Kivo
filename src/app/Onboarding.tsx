import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, type ModelProgress, type Snapshot } from "../lib/api";
import { Mark } from "../lib/mark";

/**
 * The hardest ninety seconds in the product: Kivo needs two permissions that
 * look alarming out of context, plus a large download. Every step explains
 * itself *before* triggering anything, and the last step is not skippable —
 * a user who has not succeeded once will not succeed later.
 */
export function Onboarding({ snap, onDone }: { snap: Snapshot; onDone: () => void }) {
  const [step, setStep] = useState(0);
  const [progress, setProgress] = useState<string | null>(null);
  const [failed, setFailed] = useState<string | null>(null);
  const [installed, setInstalled] = useState(snap.model_installed);
  const [confirmSkip, setConfirmSkip] = useState(false);
  const [micRequested, setMicRequested] = useState(false);
  const [micBusy, setMicBusy] = useState(false);
  const [axGranted, setAxGranted] = useState(snap.accessibility);
  const [axRequested, setAxRequested] = useState(false);

  useEffect(() => {
    const un = listen<ModelProgress>("kivo://model", (e) => {
      const p = e.payload;
      if (typeof p === "string") {
        if (p === "Done") { setProgress(null); setInstalled(true); }
        else setProgress({ Started: "Starting…", Verifying: "Verifying checksum…", Installing: "Installing…" }[p] ?? null);
      } else if ("Downloading" in p) {
        setProgress(`Downloading… ${p.Downloading.percent}%`);
      } else if ("Failed" in p) {
        setProgress(null); setFailed(p.Failed.message);
      }
    });
    return () => { un.then((f) => f()); };
  }, []);

  // While Accessibility has not been granted, poll the real state rather than
  // trusting a self-report — the permission is easy to grant in the wrong
  // place, or to grant for a build that has since been replaced.
  useEffect(() => {
    if (axGranted) return;
    const id = window.setInterval(() => {
      api.snapshot("").then((s) => { if (s.accessibility) setAxGranted(true); }).catch(() => {});
    }, 1000);
    return () => window.clearInterval(id);
  }, [axGranted]);

  const requestMic = async () => {
    setMicBusy(true);
    try {
      await api.requestMicrophone();
    } finally {
      setMicRequested(true);
      setMicBusy(false);
    }
  };

  const requestAx = async () => {
    setAxRequested(true);
    const granted = await api.requestAccessibility().catch(() => false);
    if (granted) setAxGranted(true);
    api.openAccessibility();
  };

  const steps = [
    {
      title: "Kivo turns voice into clean text.",
      body: "Hold a key, speak, let go. Punctuated text appears wherever your cursor already is — in any app. Everything runs on this Mac.",
      action: <button className="btn" onClick={() => setStep(1)}>Continue</button>,
    },
    {
      title: "It needs your microphone.",
      body: "Kivo opens the microphone only while you hold the key, and closes it the moment you let go — so the indicator in your menu bar tells the truth. Audio is never uploaded and never written to disk.",
      action: (
        <div className="stack">
          <button className="btn" disabled={micBusy} onClick={requestMic}>
            {micBusy ? "Requesting…" : micRequested ? "Requested — Continue" : "Enable microphone access"}
          </button>
          {micRequested && (
            <p className="ok">If macOS asked to allow access, choose Allow, then continue.</p>
          )}
          <button className="btn sand" onClick={() => setStep(2)}>Continue</button>
        </div>
      ),
    },
    {
      title: "And permission to type for you.",
      body: "Accessibility permission is how Kivo sees your hotkey and puts text at your cursor. It reads nothing from your screen — that code is public, in src-tauri/src/inject.",
      action: axGranted ? (
        <div className="stack">
          <p className="ok">Granted.</p>
          <button className="btn" onClick={() => setStep(3)}>Continue</button>
        </div>
      ) : (
        <div className="stack">
          <button className="btn" onClick={requestAx}>Open System Settings</button>
          {axRequested && <p className="bad">Not granted yet — checking…</p>}
          <button className="btn sand" onClick={() => setStep(3)}>I've granted it</button>
        </div>
      ),
    },
    {
      title: "Now the speech model.",
      body: `A ${snap.model_download_mb} MB download, once. Kivo checks it against a pinned SHA-256 and refuses to install anything that does not match. This is the only time Kivo touches the network.`,
      action: installed ? (
        <div className="stack">
          <p className="ok">Model installed and verified.</p>
          <button className="btn" onClick={() => setStep(4)}>Continue</button>
        </div>
      ) : (
        <div className="stack">
          <button className="btn" disabled={!!progress} onClick={() => { setFailed(null); setConfirmSkip(false); api.installModel(); }}>
            {progress ?? "Download the model"}
          </button>
          {failed && <p className="bad">{failed}</p>}
          {!confirmSkip ? (
            <button className="btn sand" onClick={() => setConfirmSkip(true)}>Skip for now</button>
          ) : (
            <>
              <p className="bad">Without a model, holding your key will not produce any text. You can install one later from Settings.</p>
              <button className="btn sand" onClick={() => setStep(4)}>Skip anyway</button>
            </>
          )}
        </div>
      ),
    },
    {
      title: "Try it.",
      body: "Click into the box below, hold your key, and say something. If nothing happens, Accessibility permission is the usual reason.",
      action: (
        <div className="stack">
          {!installed && (
            <p className="bad">
              No speech model is installed, so dictation will not produce text yet.
              Install one from Settings whenever you are ready.
            </p>
          )}
          {!axGranted && (
            <p className="bad">Accessibility is not granted yet — the hotkey will not be seen until it is.</p>
          )}
          <textarea className="tryit" rows={3} placeholder="Hold your key and speak…" />
          <button className="btn" onClick={() => { api.finishOnboarding().then(onDone); }}>
            Done — open Settings
          </button>
        </div>
      ),
    },
  ];

  const s = steps[step]!;
  return (
    <div className="onboard">
      <Mark height={26} />
      <div className="onboard__dots" aria-hidden="true">
        {steps.map((_, i) => <i key={i} data-on={i <= step} />)}
      </div>
      <h1>{s.title}</h1>
      <p>{s.body}</p>
      <div className="onboard__action">{s.action}</div>
      <p className="onboard__step">Step {step + 1} of {steps.length}</p>
    </div>
  );
}

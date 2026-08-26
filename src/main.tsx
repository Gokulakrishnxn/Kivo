import { createRoot } from "react-dom/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Mark } from "./lib/mark";
import "./styles.css";

type Mode = "standard" | "message" | "email" | "notes" | "code" | "raw";

interface Config {
  hotkey: string;
  mode: Mode;
  remove_fillers: boolean;
  remove_openers: boolean;
  collapse_stutters: boolean;
  resolve_corrections: boolean;
  replacements: { from: string; to: string }[];
  use_typing: boolean;
  keep_history: boolean;
  history_limit: number;
  launch_at_login: boolean;
}

interface Entry { raw: string; text: string; mode: string; millis: number; at: string }

interface Snapshot {
  state: "idle" | "recording" | "working";
  config: Config;
  model: string;
  model_installed: boolean;
  model_dir: string;
  history: Entry[];
  error: string | null;
  hotkeys: { value: string; label: string }[];
  modes: { value: Mode; label: string }[];
}

const RULES: { key: keyof Config; title: string; hint: string }[] = [
  { key: "remove_fillers", title: "Remove fillers", hint: "“um”, “uh”, “er” and friends" },
  { key: "remove_openers", title: "Remove openers", hint: "“so”, “well”, “basically” at the start of a sentence" },
  { key: "collapse_stutters", title: "Collapse stutters", hint: "“the the” becomes “the”" },
  { key: "resolve_corrections", title: "Resolve corrections", hint: "“Friday — no wait, Thursday” becomes “Thursday”" },
];

function App() {
  const [snap, setSnap] = useState<Snapshot | null>(null);
  const [permission, setPermission] = useState<string | null>(null);
  const [sample, setSample] = useState("");
  const [preview, setPreview] = useState("");

  const refresh = useCallback(async () => setSnap(await invoke<Snapshot>("snapshot")), []);

  useEffect(() => {
    refresh();
    invoke<{ said: string; typed: string }>("demo").then((d) => setSample(d.said));
    const subs = [
      listen<string>("kivo://state", () => refresh()),
      listen<Entry>("kivo://inserted", () => refresh()),
      listen<string>("kivo://error", () => refresh()),
      listen<string>("kivo://permission", (e) => setPermission(e.payload)),
    ];
    return () => { subs.forEach((s) => s.then((f) => f())); };
  }, [refresh]);

  const config = snap?.config;

  // Preview runs on every change so a rule can be judged before it is trusted.
  useEffect(() => {
    if (!config || !sample) return;
    invoke<string>("preview_clean", { text: sample, config }).then(setPreview);
  }, [config, sample]);

  const update = useCallback(
    async (patch: Partial<Config>) => {
      if (!snap) return;
      const next = { ...snap.config, ...patch };
      setSnap({ ...snap, config: next });
      await invoke("save_config", { config: next });
    },
    [snap],
  );

  const stateLabel = useMemo(() => {
    if (!snap) return "Starting";
    return { idle: "Ready", recording: "Listening", working: "Working" }[snap.state];
  }, [snap]);

  if (!snap || !config) return <div className="shell"><p className="empty">Starting…</p></div>;

  return (
    <>
      <div className="titlebar" />
      <div className="shell">
        <header className="head">
          {/* The lockup already contains the wordmark — a heading beside it
              would render "Kivo Kivo". */}
          <Mark height={24} />
          <h1 className="visually-hidden">Kivo</h1>
          <span className="pill" data-state={snap.state}><i />{stateLabel}</span>
        </header>

        {permission && (
          <div className="banner warn">
            <span>{permission}</span>
          </div>
        )}

        {!snap.model_installed && (
          <div className="banner warn">
            <span>
              No speech model is installed, so Kivo cannot transcribe yet. Place the
              Parakeet ONNX files in <code>{snap.model_dir}</code> and reload.
            </span>
            <button className="btn" onClick={() => invoke("reload_model").then(refresh).catch(() => refresh())}>
              Reload
            </button>
          </div>
        )}

        {snap.error && (
          <div className="banner info">
            <span>{snap.error}</span>
            <button className="btn sand" onClick={() => invoke("dismiss_error").then(refresh)}>Dismiss</button>
          </div>
        )}

        <section className="card">
          <h2>Dictation</h2>
          <p className="sub">Hold the key, speak, let go. Text lands where your cursor is.</p>
          <div className="row">
            <div className="grow"><strong>Hold-to-talk key</strong><span className="hint">A bare modifier cannot collide with an app shortcut.</span></div>
            <select value={config.hotkey} onChange={(e) => update({ hotkey: e.target.value })}>
              {snap.hotkeys.map((h) => <option key={h.value} value={h.value}>{h.label}</option>)}
            </select>
          </div>
          <div className="row">
            <div className="grow"><strong>Mode</strong><span className="hint">Changes only the cleanup, never the recognition.</span></div>
          </div>
          <div className="seg">
            {snap.modes.map((m) => (
              <button key={m.value} aria-pressed={config.mode === m.value} onClick={() => update({ mode: m.value })}>
                {m.label}
              </button>
            ))}
          </div>
          <div className="row" style={{ marginTop: 12 }}>
            <div className="grow"><strong>Insert by typing</strong><span className="hint">Slower, but it works where a synthetic paste does not.</span></div>
            <input className="switch" type="checkbox" checked={config.use_typing}
                   onChange={(e) => update({ use_typing: e.target.checked })} />
          </div>
        </section>

        <section className="card">
          <h2>Cleanup</h2>
          <p className="sub">Every rule is independent, and Raw mode bypasses all of them.</p>
          {RULES.map((r) => (
            <div className="row" key={r.key}>
              <div className="grow"><strong>{r.title}</strong><span className="hint">{r.hint}</span></div>
              <input className="switch" type="checkbox"
                     disabled={config.mode === "raw"}
                     checked={Boolean(config[r.key])}
                     onChange={(e) => update({ [r.key]: e.target.checked } as Partial<Config>)} />
            </div>
          ))}
          <div className="preview" style={{ marginTop: 14 }}>
            <div className="label">What you said</div>
            <div className="said">{sample}</div>
            <div className="label" style={{ marginTop: 12 }}>What Kivo would type</div>
            <div className="typed">{preview || "—"}</div>
          </div>
        </section>

        <section className="card">
          <h2>History</h2>
          <p className="sub">Text only, never audio. Turn it off and nothing is written to disk.</p>
          <div className="row">
            <div className="grow"><strong>Keep history</strong><span className="hint">The last {config.history_limit} dictations.</span></div>
            <input className="switch" type="checkbox" checked={config.keep_history}
                   onChange={(e) => update({ keep_history: e.target.checked })} />
          </div>
          {snap.history.length === 0
            ? <p className="empty">Nothing yet. Hold your key and say something.</p>
            : (
              <>
                {snap.history.slice(0, 8).map((e, i) => (
                  <div className="entry" key={i}>
                    <div>{e.text}</div>
                    <div className="meta">{e.mode.toLowerCase()} · {e.millis} ms</div>
                  </div>
                ))}
                <button className="btn sand" style={{ marginTop: 12 }}
                        onClick={() => invoke("clear_history").then(refresh)}>
                  Clear history
                </button>
              </>
            )}
        </section>

        <section className="card">
          <h2>Privacy</h2>
          <p className="sub">Empty is the normal state.</p>
          <div className="row"><div className="grow"><strong>Audio uploaded</strong></div><span>0 bytes</span></div>
          <div className="row"><div className="grow"><strong>Transcripts uploaded</strong></div><span>0 bytes</span></div>
          <div className="row"><div className="grow"><strong>Telemetry</strong></div><span>not implemented</span></div>
          <div className="row"><div className="grow"><strong>Account</strong></div><span>none</span></div>
          <div className="row"><div className="grow"><strong>Recognition</strong></div><span>{snap.model}</span></div>
        </section>
      </div>
    </>
  );
}

createRoot(document.getElementById("root")!).render(<App />);

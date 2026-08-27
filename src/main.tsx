import { createRoot } from "react-dom/client";
import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Mark } from "./lib/mark";
import { Onboarding } from "./app/Onboarding";
import { ago, api, type Config, type ModelProgress, type Snapshot } from "./lib/api";
import "./styles.css";

const RULES: { key: keyof Config; title: string; hint: string }[] = [
  { key: "remove_fillers", title: "Remove fillers", hint: "“um”, “uh”, “er” and friends" },
  { key: "remove_openers", title: "Remove openers", hint: "“so”, “well”, “basically” at the start of a sentence" },
  { key: "collapse_stutters", title: "Collapse stutters", hint: "“the the” becomes “the”" },
  { key: "resolve_corrections", title: "Resolve corrections", hint: "“Friday — no wait, Thursday” becomes “Thursday”" },
];

function App() {
  const [snap, setSnap] = useState<Snapshot | null>(null);
  const [showOnboarding, setShowOnboarding] = useState<boolean | null>(null);
  const [permission, setPermission] = useState<string | null>(null);
  const [sample, setSample] = useState("");
  const [preview, setPreview] = useState("");
  const [query, setQuery] = useState("");
  const [modelBusy, setModelBusy] = useState<string | null>(null);
  const [newWord, setNewWord] = useState("");
  const [newFrom, setNewFrom] = useState("");
  const [newTo, setNewTo] = useState("");

  const refresh = useCallback(async (q = query) => setSnap(await api.snapshot(q)), [query]);

  useEffect(() => {
    api.snapshot("").then((s) => { setSnap(s); setShowOnboarding(!s.onboarded); });
    api.demo().then((d) => setSample(d.said));
    const subs = [
      listen("kivo://state", () => refresh()),
      listen("kivo://inserted", () => refresh()),
      listen("kivo://error", () => refresh()),
      listen<string>("kivo://permission", (e) => setPermission(e.payload)),
      listen<ModelProgress>("kivo://model", (e) => {
        const p = e.payload;
        if (typeof p === "string") {
          if (p === "Done") { setModelBusy(null); refresh(); }
          else setModelBusy({ Started: "Starting…", Verifying: "Verifying…", Installing: "Installing…" }[p] ?? null);
        } else if ("Downloading" in p) setModelBusy(`Downloading… ${p.Downloading.percent}%`);
        else if ("Failed" in p) { setModelBusy(null); refresh(); }
      }),
    ];
    return () => { subs.forEach((s) => s.then((f) => f())); };
  }, [refresh]);

  const config = snap?.config;

  useEffect(() => {
    if (!config || !sample) return;
    api.previewClean(sample, config).then(setPreview);
  }, [config, sample]);

  const update = useCallback(async (patch: Partial<Config>) => {
    if (!snap) return;
    const next = { ...snap.config, ...patch };
    setSnap({ ...snap, config: next });
    await api.saveConfig(next);
    refresh();
  }, [snap, refresh]);

  if (!snap || !config || showOnboarding === null) {
    return <div className="shell"><p className="empty">Starting…</p></div>;
  }
  if (showOnboarding) {
    return <Onboarding snap={snap} onDone={() => { setShowOnboarding(false); refresh(); }} />;
  }

  const stateLabel = { idle: "Ready", recording: "Listening", working: "Working" }[snap.state];
  const hotkeyNote = snap.hotkeys.find((h) => h.value === config.hotkey)?.note ?? null;

  return (
    <>
      <div className="titlebar" />
      <div className="shell">
        <header className="head">
          {/* The lockup already contains the wordmark. */}
          <Mark height={24} />
          <h1 className="visually-hidden">Kivo</h1>
          <span className="pill" data-state={snap.state}><i />{stateLabel}</span>
        </header>

        {permission && (
          <div className="banner warn">
            <span>{permission}</span>
            <button className="btn" onClick={() => api.openAccessibility()}>Open Settings</button>
          </div>
        )}

        {snap.error && (
          <div className="banner info">
            <span>{snap.error}</span>
            <button className="btn sand" onClick={() => api.dismissError().then(() => refresh())}>Dismiss</button>
          </div>
        )}

        {/* ── speech model ─────────────────────────────────────────────── */}
        <section className="card">
          <h2>Speech model</h2>
          <p className="sub">Downloaded once, verified against a pinned checksum, then never contacted again.</p>
          <div className="row">
            <div className="grow">
              <strong>{snap.model_installed ? "Parakeet TDT 0.6B" : "Not installed"}</strong>
              <span className="hint">{snap.model_installed ? snap.model : `${snap.model_download_mb} MB download`}</span>
            </div>
            {snap.model_installed
              ? <button className="btn sand" onClick={() => api.removeModel().then(() => refresh())}>Remove</button>
              : <button className="btn" disabled={!!modelBusy} onClick={() => api.installModel()}>{modelBusy ?? "Download"}</button>}
          </div>
        </section>

        {/* ── dictation ────────────────────────────────────────────────── */}
        <section className="card">
          <h2>Dictation</h2>
          <p className="sub">Hold to talk, or tap twice to keep it on. Esc throws the recording away.</p>
          <div className="row">
            <div className="grow">
              <strong>Hold-to-talk key</strong>
              <span className="hint">
                A bare modifier does nothing on its own, so it cannot collide with an
                app shortcut. Space is offered only as a chord — on its own it would
                type into whatever you are working in.
              </span>
            </div>
            <select value={config.hotkey} onChange={(e) => update({ hotkey: e.target.value })}>
              {snap.hotkeys.map((h) => <option key={h.value} value={h.value}>{h.label}</option>)}
            </select>
          </div>
          {hotkeyNote && (
            <p className="note">{hotkeyNote}</p>
          )}
          <div className="row">
            <div className="grow"><strong>Microphone</strong><span className="hint">Falls back to the system default if this one is unplugged.</span></div>
            <select value={config.input_device} onChange={(e) => update({ input_device: e.target.value })}>
              <option value="">System default</option>
              {snap.devices.map((d) => <option key={d} value={d}>{d}</option>)}
            </select>
          </div>
          <div className="row"><div className="grow"><strong>Mode</strong><span className="hint">Changes only the cleanup, never the recognition.</span></div></div>
          <div className="seg">
            {snap.modes.map((m) => (
              <button key={m.value} aria-pressed={config.mode === m.value} onClick={() => update({ mode: m.value })}>{m.label}</button>
            ))}
          </div>
          <div className="row" style={{ marginTop: 12 }}>
            <div className="grow"><strong>Insert by typing</strong><span className="hint">Slower, but it works where a synthetic paste does not.</span></div>
            <input className="switch" type="checkbox" checked={config.use_typing} onChange={(e) => update({ use_typing: e.target.checked })} />
          </div>
        </section>

        {/* ── cleanup ──────────────────────────────────────────────────── */}
        <section className="card">
          <h2>Cleanup</h2>
          <p className="sub">Every rule is independent, and Raw mode bypasses all of them.</p>
          {RULES.map((r) => (
            <div className="row" key={r.key}>
              <div className="grow"><strong>{r.title}</strong><span className="hint">{r.hint}</span></div>
              <input className="switch" type="checkbox" disabled={config.mode === "raw"}
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

        {/* ── vocabulary ───────────────────────────────────────────────── */}
        <section className="card">
          <h2>Your words</h2>
          <p className="sub">Names, acronyms and jargon that generic models get wrong.</p>

          <div className="row"><div className="grow"><strong>Bias recognition</strong><span className="hint">Makes Kivo likelier to hear these at all. Reloads the model when changed.</span></div></div>
          <div className="chips">
            {config.vocabulary.map((w) => (
              <span className="chip" key={w}>{w}
                <button aria-label={`Remove ${w}`} onClick={() => update({ vocabulary: config.vocabulary.filter((x) => x !== w) })}>×</button>
              </span>
            ))}
            {config.vocabulary.length === 0 && <span className="empty">Nothing yet.</span>}
          </div>
          <form className="inline" onSubmit={(e) => {
            e.preventDefault();
            const w = newWord.trim();
            if (w && !config.vocabulary.includes(w)) update({ vocabulary: [...config.vocabulary, w] });
            setNewWord("");
          }}>
            <input type="text" placeholder="Kubernetes" value={newWord} onChange={(e) => setNewWord(e.target.value)} />
            <button className="btn sand" type="submit">Add</button>
          </form>

          <div className="row" style={{ marginTop: 8 }}><div className="grow"><strong>Replacements</strong><span className="hint">Rewrites text after recognition. Spoken form on the left.</span></div></div>
          {config.replacements.map((r, i) => (
            <div className="row" key={i}>
              <div className="grow"><strong>{r.from}</strong><span className="hint">becomes “{r.to}”</span></div>
              <button className="btn sand" onClick={() => update({ replacements: config.replacements.filter((_, j) => j !== i) })}>Remove</button>
            </div>
          ))}
          <form className="inline" onSubmit={(e) => {
            e.preventDefault();
            const from = newFrom.trim(), to = newTo.trim();
            if (from && to) update({ replacements: [...config.replacements, { from, to }] });
            setNewFrom(""); setNewTo("");
          }}>
            <input type="text" placeholder="cuber netties" value={newFrom} onChange={(e) => setNewFrom(e.target.value)} />
            <input type="text" placeholder="Kubernetes" value={newTo} onChange={(e) => setNewTo(e.target.value)} />
            <button className="btn sand" type="submit">Add</button>
          </form>
        </section>

        {/* ── history ──────────────────────────────────────────────────── */}
        <section className="card">
          <h2>History</h2>
          <p className="sub">Text only, never audio. Turn it off and the file is deleted.</p>
          <div className="row">
            <div className="grow"><strong>Keep history</strong><span className="hint">The last {config.history_limit} dictations, on this Mac only.</span></div>
            <input className="switch" type="checkbox" checked={config.keep_history} onChange={(e) => update({ keep_history: e.target.checked })} />
          </div>
          {config.keep_history && (
            <>
              <form className="inline" onSubmit={(e) => e.preventDefault()}>
                <input type="text" placeholder="Search your history…" value={query}
                       onChange={(e) => { setQuery(e.target.value); refresh(e.target.value); }} />
              </form>
              {snap.history.length === 0
                ? <p className="empty">{query ? "Nothing matches." : "Nothing yet. Hold your key and say something."}</p>
                : (
                  <>
                    {snap.history.map((e) => (
                      <div className="entry" key={e.id}>
                        <div>{e.text}</div>
                        <div className="entry__foot">
                          <span className="meta">{e.mode.toLowerCase()} · {e.millis} ms · {ago(e.at)}</span>
                          <button className="linkish" onClick={() => navigator.clipboard.writeText(e.text)}>Copy</button>
                        </div>
                      </div>
                    ))}
                    <button className="btn sand" style={{ marginTop: 12 }} onClick={() => api.clearHistory().then(() => refresh())}>Clear history</button>
                  </>
                )}
            </>
          )}
        </section>

        {/* ── privacy ──────────────────────────────────────────────────── */}
        <section className="card">
          <h2>Network</h2>
          <p className="sub">Every outbound request Kivo has made, this session and last.</p>
          {snap.network.length === 0
            ? <p className="empty">No outbound requests. This is the normal state.</p>
            : snap.network.map((n, i) => (
              <div className="row" key={i}>
                <div className="grow"><strong>{n.outcome}</strong><span className="hint">{n.url}</span></div>
                <span className="meta">{n.bytes ? `${Math.round(n.bytes / 1048576)} MB` : "—"}</span>
              </div>
            ))}
          <div className="row"><div className="grow"><strong>Audio uploaded</strong></div><span className="meta">0 bytes</span></div>
          <div className="row"><div className="grow"><strong>Transcripts uploaded</strong></div><span className="meta">0 bytes</span></div>
          <div className="row"><div className="grow"><strong>Telemetry</strong></div><span className="meta">not implemented</span></div>
          <div className="row"><div className="grow"><strong>Account</strong></div><span className="meta">none</span></div>
        </section>
      </div>
    </>
  );
}

createRoot(document.getElementById("root")!).render(<App />);

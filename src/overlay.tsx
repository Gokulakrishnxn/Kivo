import { createRoot } from "react-dom/client";
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import "./styles.css";

const BARS = [14, 22, 10, 18, 8, 20, 12];

/**
 * The dumbest window in the system: it never takes focus, never accepts input,
 * and exists so a dead microphone is visible in the first half-second rather
 * than after you have finished a paragraph.
 */
function Overlay() {
  const [state, setState] = useState<string>("recording");

  useEffect(() => {
    const un = listen<string>("kivo://state", (e) => setState(e.payload));
    return () => { un.then((f) => f()); };
  }, []);

  const working = state === "working";

  return (
    <div className="hud">
      <span className="wave" aria-hidden="true">
        {BARS.map((h, i) => (
          <i key={i} style={{ ["--h" as string]: `${h}px`, animationDelay: `${i * 0.08}s`,
                              animationPlayState: working ? "paused" : "running" }} />
        ))}
      </span>
      <span className="label">{working ? "Working…" : "Listening"}</span>
      <span className="key">hold to talk</span>
    </div>
  );
}

createRoot(document.getElementById("overlay")!).render(<Overlay />);

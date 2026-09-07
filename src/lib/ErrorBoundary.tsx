import { Component, type ReactNode } from "react";

/**
 * The alternative to a black window.
 *
 * If anything in the settings tree throws during render, React would
 * otherwise unmount the whole tree and leave a bare, unstyled document — which
 * is indistinguishable, to someone looking at it, from the app being dead.
 * This catches that and says so instead, with the one piece of information
 * that actually helps: where the log file is.
 */
export class ErrorBoundary extends Component<
  { children: ReactNode },
  { error: Error | null }
> {
  state = { error: null as Error | null };

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  componentDidCatch(error: Error, info: { componentStack?: string | null }) {
    // The renderer process has no visible console; this is the only place
    // such an error can be seen at all.
    // eslint-disable-next-line no-console
    console.error("Kivo settings crashed:", error, info.componentStack);
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div className="onboard" style={{ maxWidth: 480 }}>
        <h1>Settings hit a problem.</h1>
        <p>
          This screen failed to load. Nothing else in Kivo is affected —
          dictation keeps working from the menu bar.
        </p>
        <p className="bad" style={{ userSelect: "text" }}>
          {this.state.error.message || String(this.state.error)}
        </p>
        <p>
          The full detail is in{" "}
          <code style={{ userSelect: "text" }}>
            ~/Library/Logs/Kivo/kivo.log
          </code>
          .
        </p>
        <div className="onboard__action">
          <button className="btn" onClick={() => location.reload()}>
            Reload Settings
          </button>
        </div>
      </div>
    );
  }
}

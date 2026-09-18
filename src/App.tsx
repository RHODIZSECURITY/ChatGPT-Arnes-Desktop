import { useCallback, useEffect, useState } from "react";
import { getRuntimeStatus, startRuntime, stopRuntime } from "./runtime-client";
import { runtimeReadiness, stateLabel, type RuntimeStatus } from "./runtime-model";

const initialStatus: RuntimeStatus = {
  platform_supported: false,
  wsl_available: false,
  distro_installed: false,
  docker_ready: false,
  service_state: "unknown",
  detail: "Runtime status has not been queried yet.",
};

function App() {
  const [status, setStatus] = useState<RuntimeStatus>(initialStatus);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      setStatus(await getRuntimeStatus());
    } catch (value) {
      setError(String(value));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const applyAction = async (action: () => Promise<RuntimeStatus>) => {
    setBusy(true);
    setError(null);
    try {
      setStatus(await action());
    } catch (value) {
      setError(String(value));
    } finally {
      setBusy(false);
    }
  };

  const readiness = runtimeReadiness(status);

  return (
    <main className="shell">
      <header className="hero">
        <div>
          <p className="eyebrow">RHODIZ Arnes Desktop</p>
          <h1>Local runtime control plane</h1>
          <p className="subtle">
            Tauri renderer → typed Rust broker → RHODIZ-Arnes WSL2 → Docker.
          </p>
        </div>
        <span className={`readiness readiness--${readiness}`}>{readiness}</span>
      </header>

      <section className="status-grid" aria-label="Runtime status">
        <StatusCard title="Windows host" value={stateLabel(status.platform_supported)} />
        <StatusCard title="WSL2" value={stateLabel(status.wsl_available)} />
        <StatusCard title="RHODIZ-Arnes distro" value={stateLabel(status.distro_installed)} />
        <StatusCard title="Docker Engine" value={stateLabel(status.docker_ready)} />
        <StatusCard title="Arnes service" value={status.service_state.replaceAll("_", " ")} />
      </section>

      <section className="controls">
        <button type="button" disabled={busy} onClick={() => void refresh()}>
          Refresh
        </button>
        <button
          type="button"
          disabled={busy || !status.distro_installed}
          onClick={() => void applyAction(startRuntime)}
        >
          Start Arnes
        </button>
        <button
          type="button"
          disabled={busy || !status.distro_installed}
          onClick={() => void applyAction(stopRuntime)}
        >
          Stop Arnes
        </button>
      </section>

      <section className="diagnostic" aria-live="polite">
        <strong>Broker detail</strong>
        <p>{error ?? status.detail ?? "No diagnostic detail."}</p>
      </section>

      <footer>
        The renderer has no generic PowerShell, WSL, Docker, or process command API.
      </footer>
    </main>
  );
}

function StatusCard({ title, value }: { title: string; value: string }) {
  return (
    <article className="status-card">
      <span>{title}</span>
      <strong>{value}</strong>
    </article>
  );
}

export default App;

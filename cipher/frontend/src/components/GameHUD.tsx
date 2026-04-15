import type { ReactNode } from "react";

type GameHUDProps = {
  connected: boolean;
  tick: number | null;
  aliveCount: number;
  myName: string;
  onNameChange: (v: string) => void;
  onConnect: () => void;
  onDisconnect: () => void;
  error: string | null;
};

export function GameHUD({
  connected,
  tick,
  aliveCount,
  myName,
  onNameChange,
  onConnect,
  onDisconnect,
  error,
}: GameHUDProps) {
  return (
    <div className="pointer-events-none absolute inset-0 z-30 flex flex-col">
      {/* Top bar */}
      <header className="pointer-events-auto flex items-center justify-between gap-4 border-b border-[#1a2744] bg-[#0a0f1e]/90 px-4 py-3 shadow-[0_0_40px_rgba(43,142,240,0.12)] backdrop-blur-md">
        <div className="flex items-baseline gap-3">
          <h1 className="bg-gradient-to-r from-[#00e5ff] via-[#2b8ef0] to-[#a855f7] bg-clip-text font-display text-2xl font-extrabold tracking-tight text-transparent md:text-3xl">
            CIPHER
          </h1>
          <span className="hidden font-mono text-[10px] font-semibold uppercase tracking-[0.25em] text-[#2b8ef0]/80 sm:inline">
            chaos arena
          </span>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <input
            value={myName}
            onChange={(e) => onNameChange(e.target.value)}
            disabled={connected}
            maxLength={24}
            placeholder="CALLSIGN"
            className="pointer-events-auto w-36 border border-[#1a2744] bg-[#0d1526] px-3 py-2 font-mono text-xs text-[#f0f4ff] outline-none transition-[box-shadow] focus:border-[#2b8ef0] focus:shadow-[0_0_20px_rgba(43,142,240,0.25)] disabled:opacity-50 md:w-44"
          />
          {!connected ? (
            <button
              type="button"
              onClick={onConnect}
              className="pointer-events-auto border border-[#2b8ef0] bg-[#2b8ef0]/20 px-4 py-2 font-mono text-xs font-bold uppercase tracking-wider text-[#00e5ff] shadow-[0_0_24px_rgba(43,142,240,0.35)] transition hover:bg-[#2b8ef0]/35"
            >
              Deploy
            </button>
          ) : (
            <button
              type="button"
              onClick={onDisconnect}
              className="pointer-events-auto border border-[#ff5f57]/50 bg-[#ff5f57]/10 px-4 py-2 font-mono text-xs font-bold uppercase tracking-wider text-[#ff5f57] transition hover:bg-[#ff5f57]/20"
            >
              Abort link
            </button>
          )}
          <div
            className={`flex items-center gap-2 rounded border px-3 py-2 font-mono text-[10px] font-bold uppercase tracking-widest ${
              connected
                ? "border-[#28c840]/50 bg-[#28c840]/10 text-[#28c840]"
                : "border-[#1a2744] bg-[#0d1526] text-[#f0f4ff]/50"
            }`}
          >
            <span
              className={`h-2 w-2 rounded-full ${connected ? "animate-pulse bg-[#28c840] shadow-[0_0_10px_#28c840]" : "bg-[#1a2744]"}`}
            />
            {connected ? "live" : "offline"}
          </div>
        </div>
      </header>

      {/* Side panels */}
      <div className="flex flex-1 justify-between p-3 md:p-4">
        <aside className="pointer-events-none flex max-w-[11rem] flex-col gap-2 font-mono text-[10px] uppercase tracking-wider text-[#f0f4ff]/70 md:max-w-xs">
          <Panel title="Telemetry">
            <Row label="Tick" value={tick != null ? String(tick) : "—"} />
            <Row label="Active" value={String(aliveCount)} />
          </Panel>
          <Panel title="Controls">
            <p className="text-[9px] leading-relaxed normal-case text-[#f0f4ff]/55">
              WASD / arrows move · <span className="text-[#00e5ff]">Space</span> jump · open two tabs to test netcode
            </p>
          </Panel>
        </aside>

        <aside className="pointer-events-none hidden max-w-[10rem] flex-col gap-2 md:flex">
          <Panel title="Status">
            <p className="text-[9px] leading-relaxed normal-case text-[#f0f4ff]/55">
              Dev: run <code className="text-[#2b8ef0]">cargo run -p cipher-server</code> then{" "}
              <code className="text-[#2b8ef0]">npm run dev</code> here.
            </p>
          </Panel>
        </aside>
      </div>

      {error ? (
        <div className="pointer-events-auto mx-auto mb-4 max-w-lg border border-[#ff5f57]/40 bg-[#ff5f57]/10 px-4 py-2 text-center font-mono text-xs text-[#ff5f57]">
          {error}
        </div>
      ) : null}
    </div>
  );
}

function Panel({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="border border-[#1a2744] bg-[#0a0f1e]/75 p-3 shadow-[inset_0_0_30px_rgba(43,142,240,0.06)] backdrop-blur-sm">
      <div className="mb-2 font-display text-[11px] font-bold tracking-[0.2em] text-[#2b8ef0]">{title}</div>
      {children}
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between gap-4 border-b border-[#1a2744]/60 py-1 last:border-0">
      <span className="text-[#f0f4ff]/45">{label}</span>
      <span className="text-[#00e5ff]">{value}</span>
    </div>
  );
}

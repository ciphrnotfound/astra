import { useMemo, useState } from "react";
import { ArenaScene } from "./components/ArenaScene";
import { GameHUD } from "./components/GameHUD";
import { useCipherSocket } from "./hooks/useCipherSocket";

export default function App() {
  const [name, setName] = useState("player");
  const { connect, disconnect, connected, myId, gameState, lastError } = useCipherSocket();

  const players = gameState?.players ?? [];
  const tick = gameState?.tick ?? null;
  const aliveCount = useMemo(() => players.filter((p) => p.alive).length, [players]);

  return (
    <div className="cipher-crt relative h-full w-full overflow-hidden">
      <div className="cipher-vignette" />
      <div className="absolute inset-0 z-0">
        <ArenaScene players={players} myId={myId} />
      </div>
      <GameHUD
        connected={connected}
        tick={tick}
        aliveCount={aliveCount}
        myName={name}
        onNameChange={setName}
        onConnect={() => connect(name)}
        onDisconnect={disconnect}
        error={lastError}
      />
    </div>
  );
}

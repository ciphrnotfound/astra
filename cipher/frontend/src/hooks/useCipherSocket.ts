import { useCallback, useEffect, useRef, useState } from "react";

export type PlayerView = {
  id: string;
  name: string;
  x: number;
  y: number;
  color: string;
  hp: number;
  alive: boolean;
};

export type GameState = {
  tick: number;
  players: PlayerView[];
};

function wsUrl() {
  const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
  return `${proto}//${window.location.host}/ws`;
}

export function useCipherSocket() {
  const [connected, setConnected] = useState(false);
  const [myId, setMyId] = useState<string | null>(null);
  const [arena, setArena] = useState({ w: 800, h: 800 });
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [lastError, setLastError] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const keysRef = useRef<Set<string>>(new Set());

  const send = useCallback((msg: unknown) => {
    const w = wsRef.current;
    if (w && w.readyState === WebSocket.OPEN) {
      w.send(JSON.stringify(msg));
    }
  }, []);

  const sendInput = useCallback(() => {
    const k = keysRef.current;
    let dx = 0;
    let dy = 0;
    if (k.has("a") || k.has("arrowleft")) dx -= 1;
    if (k.has("d") || k.has("arrowright")) dx += 1;
    if (k.has("w") || k.has("arrowup")) dy -= 1;
    if (k.has("s") || k.has("arrowdown")) dy += 1;
    const jump = k.has(" ") || k.has("space");
    send({ type: "input", dx, dy, jump });
  }, [send]);

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      keysRef.current.add(e.key.toLowerCase());
      if (e.key === " ") e.preventDefault();
      if (wsRef.current?.readyState === WebSocket.OPEN && myId) sendInput();
    };
    const up = (e: KeyboardEvent) => {
      keysRef.current.delete(e.key.toLowerCase());
      if (wsRef.current?.readyState === WebSocket.OPEN && myId) sendInput();
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
  }, [myId, sendInput]);

  useEffect(() => {
    const id = window.setInterval(() => {
      if (wsRef.current?.readyState === WebSocket.OPEN && myId) sendInput();
    }, 1000 / 30);
    return () => clearInterval(id);
  }, [myId, sendInput]);

  const connect = useCallback(
    (name: string) => {
      if (wsRef.current?.readyState === WebSocket.OPEN) return;
      setLastError(null);
      const ws = new WebSocket(wsUrl());
      wsRef.current = ws;
      ws.onopen = () => {
        setConnected(true);
        send({ type: "join", name: name.trim() || "player" });
      };
      ws.onclose = () => {
        setConnected(false);
        setMyId(null);
        wsRef.current = null;
      };
      ws.onerror = () => {
        setLastError("WebSocket error — is cipher-server running on :3847?");
      };
      ws.onmessage = (ev) => {
        try {
          const msg = JSON.parse(ev.data as string) as Record<string, unknown>;
          const t = msg.type as string;
          if (t === "welcome") {
            setMyId(msg.id as string);
            setArena({
              w: msg.arena_w as number,
              h: msg.arena_h as number,
            });
          } else if (t === "state") {
            setGameState({
              tick: msg.tick as number,
              players: msg.players as PlayerView[],
            });
          } else if (t === "error") {
            setLastError(msg.message as string);
          }
        } catch {
          /* ignore */
        }
      };
    },
    [send],
  );

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
    setConnected(false);
    setMyId(null);
  }, []);

  return {
    connect,
    disconnect,
    connected,
    myId,
    arena,
    gameState,
    lastError,
  };
}

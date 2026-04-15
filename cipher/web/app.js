(() => {
  const canvas = document.getElementById("c");
  const ctx = canvas.getContext("2d");
  const nameInput = document.getElementById("name");
  const connectBtn = document.getElementById("connect");
  const statusEl = document.getElementById("status");
  const hudEl = document.getElementById("hud");

  let ws = null;
  let myId = null;
  let arena = { w: 800, h: 800 };
  /** @type {{ tick: number, players: Array<{id:string,name:string,x:number,y:number,color:string,hp:number,alive:boolean}> }} */
  let lastState = { tick: 0, players: [] };

  const keys = new Set();

  function wsUrl() {
    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    return `${proto}//${location.host}/ws`;
  }

  function send(msg) {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(msg));
    }
  }

  function sendInput() {
    let dx = 0;
    let dy = 0;
    if (keys.has("a") || keys.has("arrowleft")) dx -= 1;
    if (keys.has("d") || keys.has("arrowright")) dx += 1;
    if (keys.has("w") || keys.has("arrowup")) dy -= 1;
    if (keys.has("s") || keys.has("arrowdown")) dy += 1;
    const jump = keys.has(" ") || keys.has("space");
    send({ type: "input", dx, dy, jump });
  }

  window.addEventListener("keydown", (e) => {
    const k = e.key.toLowerCase();
    keys.add(k);
    if (k === " ") e.preventDefault();
    sendInput();
  });
  window.addEventListener("keyup", (e) => {
    keys.delete(e.key.toLowerCase());
    sendInput();
  });

  const inputInterval = setInterval(() => {
    if (ws && ws.readyState === WebSocket.OPEN && myId) sendInput();
  }, 1000 / 30);

  function draw() {
    ctx.fillStyle = "#0a0f1e";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.strokeStyle = "#1a2744";
    ctx.lineWidth = 4;
    ctx.strokeRect(2, 2, arena.w - 4, arena.h - 4);

    for (const p of lastState.players) {
      if (!p.alive) continue;
      ctx.beginPath();
      ctx.arc(p.x, p.y, 30, 0, Math.PI * 2);
      ctx.fillStyle = p.color;
      ctx.fill();
      ctx.strokeStyle = "#0a0f1e";
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.fillStyle = "#f0f4ff";
      ctx.font = "12px system-ui";
      ctx.textAlign = "center";
      ctx.fillText(p.name, p.x, p.y - 38);
      const barW = 50;
      ctx.fillStyle = "#1a2744";
      ctx.fillRect(p.x - barW / 2, p.y - 32, barW, 4);
      ctx.fillStyle = p.hp > 30 ? "#28c840" : "#ff5f57";
      ctx.fillRect(p.x - barW / 2, p.y - 32, (barW * p.hp) / 100, 4);
      if (p.id === myId) {
        ctx.strokeStyle = "#2b8ef0";
        ctx.lineWidth = 3;
        ctx.beginPath();
        ctx.arc(p.x, p.y, 34, 0, Math.PI * 2);
        ctx.stroke();
      }
    }

    requestAnimationFrame(draw);
  }
  requestAnimationFrame(draw);

  connectBtn.addEventListener("click", () => {
    if (ws && ws.readyState === WebSocket.OPEN) return;
    statusEl.textContent = "Connecting…";
    connectBtn.disabled = true;
    ws = new WebSocket(wsUrl());
    ws.onopen = () => {
      statusEl.textContent = "Connected";
      send({ type: "join", name: nameInput.value || "player" });
    };
    ws.onclose = () => {
      statusEl.textContent = "Disconnected";
      connectBtn.disabled = false;
      myId = null;
    };
    ws.onerror = () => {
      statusEl.textContent = "Error";
      connectBtn.disabled = false;
    };
    ws.onmessage = (ev) => {
      try {
        const msg = JSON.parse(ev.data);
        if (msg.type === "welcome") {
          myId = msg.id;
          arena.w = msg.arena_w;
          arena.h = msg.arena_h;
        } else if (msg.type === "state") {
          lastState = msg;
          hudEl.textContent = `Tick: ${msg.tick} · Players: ${msg.players.filter((p) => p.alive).length}`;
        } else if (msg.type === "error") {
          statusEl.textContent = msg.message;
        }
      } catch {
        /* ignore */
      }
    };
  });
})();

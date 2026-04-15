use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use cipher_protocol::{ClientMessage, PlayerView, ServerMessage};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{broadcast, RwLock};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

const ARENA_W: f32 = 800.0;
const ARENA_H: f32 = 800.0;
const PLAYER_R: f32 = 30.0;
const TICK_HZ: u64 = 20;
const DT: f32 = 1.0 / TICK_HZ as f32;

const COLORS: &[&str] = &[
    "#00e5ff", "#00ff88", "#ff9f43", "#ff5f9e",
    "#feca57", "#a55eea", "#ff5f57", "#ffffff",
];

#[derive(Clone)]
struct Player {
    name: String,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    color: String,
    hp: f32,
    alive: bool,
    input_dx: i8,
    input_dy: i8,
    jump_held: bool,
}

struct GameInner {
    tick: u64,
    players: HashMap<Uuid, Player>,
}

impl Default for GameInner {
    fn default() -> Self {
        Self {
            tick: 0,
            players: HashMap::new(),
        }
    }
}

#[derive(Clone)]
struct Hub {
    inner: Arc<RwLock<GameInner>>,
    tx: broadcast::Sender<String>,
}

impl Hub {
    fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            inner: Arc::new(RwLock::new(GameInner::default())),
            tx,
        }
    }

    fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    async fn join(&self, name: String) -> Uuid {
        let id = Uuid::new_v4();
        let idx = {
            let g = self.inner.read().await;
            g.players.len() % COLORS.len()
        };
        let color = COLORS[idx].to_string();
        let (sx, sy) = spawn_corner(self.inner.read().await.players.len());
        let mut g = self.inner.write().await;
        g.players.insert(
            id,
            Player {
                name: name.trim().to_string().chars().take(24).collect(),
                x: sx,
                y: sy,
                vx: 0.0,
                vy: 0.0,
                color,
                hp: 100.0,
                alive: true,
                input_dx: 0,
                input_dy: 0,
                jump_held: false,
            },
        );
        id
    }

    async fn leave(&self, id: Uuid) {
        let mut g = self.inner.write().await;
        g.players.remove(&id);
    }

    async fn set_input(&self, id: Uuid, dx: i8, dy: i8, jump: bool) {
        let mut g = self.inner.write().await;
        if let Some(p) = g.players.get_mut(&id) {
            if !p.alive {
                return;
            }
            p.input_dx = dx.clamp(-1, 1);
            p.input_dy = dy.clamp(-1, 1);
            p.jump_held = jump;
        }
    }

    async fn tick_once(&self) {
        const GRAVITY: f32 = 1500.0;
        const ACCEL: f32 = 3200.0;
        const MAX_SPEED: f32 = 420.0;
        const FRICTION: f32 = 0.88;
        const JUMP_V: f32 = -520.0;

        let mut g = self.inner.write().await;
        g.tick += 1;

        let min = PLAYER_R;
        let max_x = ARENA_W - PLAYER_R;
        let max_y = ARENA_H - PLAYER_R;

        for p in g.players.values_mut() {
            if !p.alive {
                continue;
            }

            let ax = p.input_dx as f32 * ACCEL;
            let ay = p.input_dy as f32 * ACCEL;
            p.vx += ax * DT;
            p.vy += ay * DT;
            p.vy += GRAVITY * DT;

            p.vx *= FRICTION;
            p.vy *= FRICTION;

            let speed = (p.vx * p.vx + p.vy * p.vy).sqrt();
            if speed > MAX_SPEED {
                let s = MAX_SPEED / speed;
                p.vx *= s;
                p.vy *= s;
            }

            p.x += p.vx * DT;
            p.y += p.vy * DT;

            if p.x < min {
                p.x = min;
                p.vx *= -0.25;
            } else if p.x > max_x {
                p.x = max_x;
                p.vx *= -0.25;
            }
            if p.y < min {
                p.y = min;
                p.vy *= -0.25;
            } else if p.y > max_y {
                p.y = max_y;
                p.vy = 0.0;
                if p.jump_held {
                    p.vy = JUMP_V;
                }
            }

            if p.y > ARENA_H + 80.0 {
                p.alive = false;
                p.hp = 0.0;
            }
        }

        let players: Vec<PlayerView> = g
            .players
            .iter()
            .map(|(id, p)| PlayerView {
                id: *id,
                name: p.name.clone(),
                x: p.x,
                y: p.y,
                color: p.color.clone(),
                hp: p.hp,
                alive: p.alive,
            })
            .collect();

        let msg = ServerMessage::State {
            tick: g.tick,
            players,
        };
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = self.tx.send(json);
        }
    }
}

fn spawn_corner(index: usize) -> (f32, f32) {
    let margin = 80.0;
    let corners = [
        (margin, margin),
        (ARENA_W - margin, margin),
        (margin, ARENA_H - margin),
        (ARENA_W - margin, ARENA_H - margin),
    ];
    corners[index % corners.len()]
}

async fn game_loop(hub: Arc<Hub>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1000 / TICK_HZ));
    loop {
        interval.tick().await;
        hub.tick_once().await;
    }
}

#[derive(Clone)]
struct AppState {
    hub: Arc<Hub>,
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let hub = state.hub.clone();

    let mut player_id: Option<Uuid> = None;

    while player_id.is_none() {
        let Some(Ok(msg)) = socket.recv().await else {
            return;
        };
        let Message::Text(text) = msg else {
            continue;
        };
        let Ok(cm) = serde_json::from_str::<ClientMessage>(&text) else {
            let err = ServerMessage::Error {
                message: "invalid JSON".into(),
            };
            let _ = socket
                .send(Message::Text(serde_json::to_string(&err).unwrap()))
                .await;
            continue;
        };

        if let ClientMessage::Join { name } = cm {
            let name = if name.trim().is_empty() {
                "player".into()
            } else {
                name
            };
            let id = hub.join(name).await;
            player_id = Some(id);
            let welcome = ServerMessage::Welcome {
                id,
                arena_w: ARENA_W,
                arena_h: ARENA_H,
            };
            let _ = socket
                .send(Message::Text(
                    serde_json::to_string(&welcome).unwrap_or_default(),
                ))
                .await;
        }
    }

    let id = player_id.unwrap();

    let mut rx = hub.subscribe();
    let (mut ws_tx, mut ws_rx) = socket.split();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_tx.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = ws_rx.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(cm) = serde_json::from_str::<ClientMessage>(&text) {
                    match cm {
                        ClientMessage::Join { .. } => {}
                        ClientMessage::Input { dx, dy, jump } => {
                            hub.set_input(id, dx, dy, jump).await;
                        }
                        ClientMessage::Vote { .. } => {
                            // MVP: votes wired in next iteration
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    send_task.abort();
    hub.leave(id).await;
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let hub = Arc::new(Hub::new(256));
    let hub_loop = hub.clone();
    tokio::spawn(async move {
        game_loop(hub_loop).await;
    });

    let state = AppState { hub };

    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dist = manifest_dir.join("../frontend/dist");
    let web_dir = if dist.join("index.html").exists() {
        dist
    } else {
        manifest_dir.join("../web")
    };
    tracing::info!("Static root: {:?}", web_dir);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .nest_service("/", ServeDir::new(web_dir))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3847));
    tracing::info!("CIPHER MVP — open http://{}/", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

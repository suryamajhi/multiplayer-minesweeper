mod board;

use crate::board::Board;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Router};
use futures::{SinkExt, StreamExt, TryFutureExt};
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::services::ServeFile;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

struct AppState {
    user_set: Mutex<HashSet<String>>,
    // Channel used to send messages to all connected clients.
    tx: broadcast::Sender<String>,
    board: Mutex<Board>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let user_set = Mutex::new(HashSet::new());
    let board = Mutex::new(Board::new(10, 10));
    let (tx, _rx) = broadcast::channel(100);

    let app_state = Arc::new(AppState {
        user_set,
        tx,
        board,
    });

    let app = Router::new()
        .route("/", get(index))
        .route_service("/css", ServeFile::new("./web/style.css"))
        .route_service("/css-tailwind", ServeFile::new("./dist/output.css"))
        .route_service("/js", ServeFile::new("./web/script.js"))
        .route("/websocket", get(websocket_chat_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_chat_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();
    // Username gets set in the receive loop, if it's valid.
    let mut username = String::new();
    // Loop until a text message is found
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            // If username that is sent by client is not taken, fill username string.
            check_username(&state, &mut username, &name);

            if !username.is_empty() {
                break;
            } else {
                let _ = sender
                    .send(Message::Text(String::from("{\"message\": \"Username already taken.\"}")))
                    .await;
                return;
            }
        }
    }

    // We subscribe *before* sending the "joined" message, so that we will also
    // display it to our client.
    let mut rx = state.tx.subscribe();

    let msg = {
        let mut board = state.board.lock().unwrap();
        board.add_player();
        serde_json::to_string(&board.deref()).unwrap()
    };

    let _ = state.tx.send(msg);

    let sender = Arc::new(tokio::sync::Mutex::new(sender));

    let send_sender = sender.clone();
    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if send_sender.lock().await.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers
    let cloned_state = state.clone();
    let rcv_sender = sender.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(command))) = receiver.next().await {
            // Handle the command and return board json to clients
            let mut is_boom = false;
            let response_to_client;
            let response_to_others;

            {
                let mut board = cloned_state.board.lock().unwrap();
                let line = &command;
                {
                    let mut tokens = line.split_whitespace();
                    let command_token = tokens.next().unwrap();
                    match command_token {
                        &_ => {
                            let x = tokens.next().unwrap().parse::<i32>().unwrap();
                            let y = tokens.next().unwrap().parse::<i32>().unwrap();
                            match command_token {
                                "dig" => {
                                    let result = board.dig(y, x);
                                    if board.is_complete() {
                                        response_to_client = "{\"message\":\"WIN\"}".into();
                                        response_to_others = "{\"message\":\"WIN\"}".into();
                                        board.reset();

                                    } else if result == "BOOM" {
                                        is_boom = true;
                                        response_to_client = "{\"message\": \"BOOM\"}".into();
                                        response_to_others =serde_json::to_string(&board.deref()).unwrap();
                                    } else {
                                        response_to_client = result.clone();
                                        response_to_others = result.clone();
                                    }
                                }
                                "flag" => {
                                    let result = board.flag(y, x);
                                    response_to_client = result.clone();
                                    response_to_others = result;
                                },
                                "deflag" => {
                                    let result = board.deflag(y, x);
                                    response_to_client = result.clone();
                                    response_to_others = result;
                                },
                                &_ => {
                                    response_to_client = "".into();
                                    response_to_others = "".into();
                                }
                            }
                        }
                    }
                }
            };

            if is_boom {
                if rcv_sender.lock().await.send(Message::Text(response_to_client)).await.is_err() {
                    tracing::error!("Failed to send Game Over message to client");
                }
                return;
            }

            let _ = tx.send(response_to_others);

        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Reached here meaning, the client has left.
    let msg = {
        let mut board = state.board.lock().unwrap();
        board.remove_player();
        serde_json::to_string(&board.deref()).unwrap()
    };
    let _ = state.tx.send(msg);

    // Remove username from map so new clients can take it again.
    state.user_set.lock().unwrap().remove(&username);
}

fn check_username(state: &AppState, string: &mut String, name: &str) {
    let mut user_set = state.user_set.lock().unwrap();

    if !user_set.contains(name) {
        user_set.insert(name.to_owned());

        string.push_str(name);
    }
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}


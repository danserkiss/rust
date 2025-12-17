use super::clientset::ClientSet;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chat::{ChatMessage, Message};
use serde::Deserialize;
use std::sync::Arc;

use axum::{Json, Router, response::Response, routing::get};

struct AppState {
    set: ClientSet,
}

#[derive(Deserialize)]
struct GetPath {
    id: String,
}

#[derive(Deserialize)]
struct QuerryMsg {
    msg: String,
}
pub struct AxumServer {}
impl AxumServer {
    pub async fn start_http_server(set: ClientSet) {
        let state = AppState { set: set.clone() };
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();

        let app = Router::new()
            .route("/", get(root))
            .route("/users", get(get_users))
            .route("/users/{id}", get(private_msg_to_user))
            .route("/users/kick/{id}", get(kick_user))
            .with_state(Arc::new(state));

        axum::serve(listener, app).await.unwrap();

        async fn private_msg_to_user(
            path: Path<GetPath>,
            msg: Query<QuerryMsg>,
            state: State<Arc<AppState>>,
        ) -> Response<String> {
            let ch_msg = Message::ChatMessage(ChatMessage {
                addr: "Admin".to_string(),
                msg: msg.msg.clone(),
            });

            let Ok(ch_msg_json) = ch_msg.to_json() else {
                let mut resp = Response::new("Error serialize msg".to_string());
                *resp.status_mut() = StatusCode::CONFLICT;
                return resp;
            };

            send_to_user(ch_msg_json, path.id.clone(), state.set.clone()).await
        }
        async fn kick_user(path: Path<GetPath>, state: State<Arc<AppState>>) -> Response<String> {
            let Ok(json_kick_msg) = Message::Kick.to_json() else {
                let mut resp = Response::new("Error serialize msg".to_string());
                *resp.status_mut() = StatusCode::CONFLICT;
                return resp;
            };
            send_to_user(json_kick_msg.clone(), path.id.clone(), state.set.clone()).await
        }

        async fn get_users(state: State<Arc<AppState>>) -> Json<Vec<String>> {
            let mutex_guard = state.set.set.lock().await;
            Json(mutex_guard.iter().map(|e| e.0.to_string()).collect())
        }

        async fn root() -> &'static str {
            "Hello, World!"
        }

        async fn send_to_user(msg: String, id: String, set: ClientSet) -> Response<String> {
            let mutex_guard = set.set.lock().await;
            if mutex_guard.contains_key(&id) {
                if let Some(tx_admin) = mutex_guard.get(&id) {
                    let _ = tx_admin.send(msg.clone()).await;
                    return Response::new("Succefully send message".to_string());
                } else {
                    let mut resp = Response::new("Error sending msg".to_string());
                    *resp.status_mut() = StatusCode::NOT_FOUND;
                    return resp;
                }
            } else {
                let mut resp = Response::new("No such user found".to_string());
                *resp.status_mut() = StatusCode::CONFLICT;
                return resp;
            }
        }
    }
}

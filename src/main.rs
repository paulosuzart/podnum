use std::ops::Deref;
use std::sync::{Arc};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Router;
use axum::routing::{get, post};
use clap::Parser;
use tokio::sync::Mutex;
use tracing::debug;
use tracing::info;
use tracing_subscriber;

use config::PodNumArgs;

use crate::podnum::server::{get_omni_paxos, Server};

mod podnum;
mod config;

#[derive(Clone)]
struct AppState {
    pub server: Arc<Mutex<Server>>,
}

async fn handle_podnum(State(state): State<Arc<AppState>>,
                       Path(host): Path<String>) -> impl IntoResponse {
    let x = state.server.lock().await.get_podnum(&host);

    (StatusCode::OK, "1")
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Sample");

    let args = PodNumArgs::parse();
    info!("Pid {}!", args.pid);
    debug!("nodes {:?}", args.nodes);
    let server = get_omni_paxos(args.pid, args.nodes);
    let serverMut = Arc::new(Mutex::new(*server));
    let serverRun = serverMut.clone();
    tokio::spawn( async move { serverRun.clone().lock().await.run().await; });

    let state = Arc::new(AppState { server: serverMut });

    let app = Router::new()
        .route("/health", get(|| async {
            Response::new("OK");
        }))
        .route("/podnum/:host", post(handle_podnum))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

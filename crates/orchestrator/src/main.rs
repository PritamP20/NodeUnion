mod api;
mod model;
mod state;

use api::{build_router, run_pending_scheduler_loop, AppState};
use reqwest::Client;
use state::{OrchestratorState, SharedState };
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state: SharedState = Arc::new(RwLock::new(OrchestratorState::default()));
    let http = Client::new();

    let app_state = AppState { state, http };

    // Background scheduler retries pending jobs periodically.
    {
        let scheduler_state = app_state.clone();
        tokio::spawn(async move {
            run_pending_scheduler_loop(scheduler_state, 5).await;
        });
    }

    let app = build_router(app_state);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("orchestrator listening on 0.0.0.0:8080");
    axum::serve(listener, app).await?;
    Ok(())
}

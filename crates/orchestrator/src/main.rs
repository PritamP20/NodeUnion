mod api;
mod db;
mod model;
mod state;
mod solana_client;

use api::{
    build_router, hydrate_runtime_state_from_db, run_pending_scheduler_loop,
    run_status_maintenance_loop, AppState,
};
use db::schema::NetworkRow;
use dotenvy::dotenv;
use reqwest::Client;
use solana_client::SolanaClient;
use state::{OrchestratorState, SharedState};
use std::env;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL is required (set Neon connection string)");
    let db_pool = db::new_pool(&database_url).await?;
    db::init_schema(&db_pool).await?;

    let state: SharedState = Arc::new(RwLock::new(OrchestratorState::default()));
    let http = Client::new();
    let solana = SolanaClient::from_env()?;
    let managed_network_id = env::var("ORCHESTRATOR_NETWORK_ID")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let orchestrator_public_url = env::var("ORCHESTRATOR_PUBLIC_URL")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let managed_network_name = env::var("ORCHESTRATOR_NETWORK_NAME")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let managed_network_description = env::var("ORCHESTRATOR_NETWORK_DESCRIPTION")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    if let Some(network_id) = managed_network_id.clone() {
        let network_row = NetworkRow {
            network_id: network_id.clone(),
            name: managed_network_name.unwrap_or_else(|| network_id.clone()),
            description: managed_network_description,
            orchestrator_url: orchestrator_public_url.clone(),
            status: "Active".to_string(),
            created_at_epoch_secs: now_epoch_secs(),
        };

        db::networks_repo::create_network(&db_pool, &network_row).await?;
    }

    let app_state = AppState {
        state,
        http,
        db: db_pool,
        solana,
        managed_network_id,
        orchestrator_public_url,
    };

    hydrate_runtime_state_from_db(&app_state).await;

    // Background scheduler retries pending jobs periodically.
    {
        let scheduler_state = app_state.clone();
        tokio::spawn(async move {
            run_pending_scheduler_loop(scheduler_state, 5).await;
        });
    }

    {
        let maintenance_state = app_state.clone();
        tokio::spawn(async move {
            run_status_maintenance_loop(maintenance_state, 5, 180).await;
        });
    }

    let app = build_router(app_state);

    let bind_addr = env::var("ORCHESTRATOR_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = TcpListener::bind(&bind_addr).await?;
    println!("orchestrator listening on {}", bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

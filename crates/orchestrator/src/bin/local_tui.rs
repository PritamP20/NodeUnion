#[tokio::main]
async fn main() -> std::io::Result<()> {
    let base_url = std::env::var("ORCHESTRATOR_URL")
        .or_else(|_| std::env::var("ORCHESTRATOR_BASE_URL"))
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    nodeunion_orchestrator::dashboard::run(base_url).await
}

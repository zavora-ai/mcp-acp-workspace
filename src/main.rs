mod server;
mod store;
mod types;

use rmcp::{ServiceExt, transport::stdio};
use server::AcpServer;
use store::AcpStore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = Arc::new(AcpStore::new());
    let server = AcpServer { store };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

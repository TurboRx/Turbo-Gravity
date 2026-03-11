pub mod models;

use anyhow::Context;
use mongodb::{options::ClientOptions, Client};

/// Connect to MongoDB using the provided URI and return a client.
pub async fn connect(uri: &str) -> anyhow::Result<Client> {
    let options = ClientOptions::parse(uri)
        .await
        .context("Failed to parse MongoDB URI")?;
    let client = Client::with_options(options).context("Failed to create MongoDB client")?;
    Ok(client)
}

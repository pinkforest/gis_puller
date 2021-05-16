//! HTTP Server Mock provides httpmock MockServer instance for development testing
use httpmock::MockServer;
use httpmock::Method::GET;
use httpmock::MockRef;
use tokio::io::{self, AsyncReadExt};
use tokio::fs::File;
use std::path::Path;
use std::sync::{Arc,RwLock, RwLockReadGuard, RwLockWriteGuard};
use state::Storage;

// @NOTE regression tokio::fs: https://github.com/tokio-rs/tokio/issues/1844
/// Re-uses a mock HTTP Server from a pool using httpmock MockServer and returns it
pub async fn serve() -> Result<MockServer, Box<dyn std::error::Error>> {

  // @TODO: Make this a macro_rules!
  let server = MockServer::start_async().await;
  
  Ok(server)
}

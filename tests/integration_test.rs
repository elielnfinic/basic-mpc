use anyhow::Result;
use rust_mpc::p2p::P2pNode;
use tokio::time::{sleep, Duration};
use tracing::{info};
use tracing_subscriber;

#[tokio::test]
async fn test_p2p_node_creation() -> Result<()> {
    // Initialize tracing for the test
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();
    
    info!("Creating P2P node");
    let node = P2pNode::new(1).await?;
    
    // Verify the node was created successfully
    assert_eq!(node.party_id, 1);
    assert_eq!(node.get_connected_party_count(), 0);
    
    info!("P2P node created successfully");
    Ok(())
}

#[tokio::test]
async fn test_p2p_node_listening() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();
    
    info!("Creating and starting P2P node listener");
    let mut node = P2pNode::new(1).await?;
    
    // Start listening on a test port
    node.start_listening(0).await?; // Port 0 for automatic assignment
    
    // Give it a moment to start listening
    sleep(Duration::from_millis(100)).await;
    
    info!("P2P node is listening successfully");
    Ok(())
}
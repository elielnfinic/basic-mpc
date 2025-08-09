mod secret_sharing;
mod party;
mod p2p;

use anyhow::Result;
use futures::StreamExt;
use party::Party;
use secret_sharing::{SecretSharing, Share};
use std::env;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    let args: Vec<String> = env::args().collect();
    
    // Check if we should use p2p mode (no peer addresses provided)
    if args.len() < 3 {
        eprintln!("Usage: {} <party_id> <port> [peer1_addr] [peer2_addr] ...", args[0]);
        eprintln!("If no peer addresses are provided, p2p discovery will be used");
        std::process::exit(1);
    }
    
    let party_id: usize = args[1].parse()?;
    let port: u16 = args[2].parse()?;
    let peers = if args.len() > 3 { 
        args[3..].to_vec() 
    } else { 
        Vec::new() 
    };
    
    if peers.is_empty() {
        info!("Starting in p2p discovery mode");
        run_p2p_mode(party_id, port).await
    } else {
        info!("Starting in legacy TCP mode");
        run_tcp_mode(party_id, port, peers).await
    }
}

async fn run_p2p_mode(party_id: usize, port: u16) -> Result<()> {
    let mut p2p_node = p2p::P2pNode::new(party_id).await?;
    p2p_node.start_listening(port).await?;
    
    info!("P2P node started, waiting for peer discovery...");
    
    // Wait for some peers to connect
    loop {
        let event = p2p_node.swarm.select_next_some().await;
        p2p_node.handle_swarm_event(event).await;
        
        // Start MPC when we have enough peers
        if p2p_node.get_connected_party_count() >= 2 {
            info!("Found {} peers, starting MPC protocol", p2p_node.get_connected_party_count());
            break;
        }
    }
    
    // Run MPC protocol using p2p
    run_mpc_with_p2p(party_id, &mut p2p_node).await
}

async fn run_mpc_with_p2p(party_id: usize, p2p_node: &mut p2p::P2pNode) -> Result<()> {
    // Simulate MPC addition protocol with p2p networking
    if party_id == 1 {
        // Party 1 is the coordinator
        let a = 42;
        let b = 17;
        
        info!("Starting MPC computation: {} + {} = ?", a, b);
        
        // Split secrets
        let a_shares = SecretSharing::split_secret(a, 3);
        let b_shares = SecretSharing::split_secret(b, 3);
        
        // Send shares to other parties via p2p
        for share in a_shares.iter().chain(b_shares.iter()) {
            if share.party_id != party_id {
                p2p_node.send_share(share.party_id, share).await?;
            }
        }
    }
    
    // Keep processing network events
    loop {
        tokio::select! {
            event = p2p_node.swarm.select_next_some() => {
                p2p_node.handle_swarm_event(event).await;
            }
            share = p2p_node.shares_rx.recv() => {
                if let Some(share) = share {
                    info!("Received share from party {}: {}", share.party_id, share.value);
                    
                    // For demonstration, just acknowledge receipt
                    // In a full implementation, we'd collect shares and compute the result
                    break;
                }
            }
            _ = sleep(Duration::from_secs(30)) => {
                warn!("MPC protocol timeout");
                break;
            }
        }
    }
    
    info!("MPC protocol completed");
    Ok(())
}

async fn run_tcp_mode(party_id: usize, port: u16, peers: Vec<String>) -> Result<()> {
    let mut party = Party::new(party_id, port, peers).await?;
    
    // Simulate MPC addition protocol
    if party_id == 1 {
        // Party 1 is the coordinator in this example
        let a = 42;
        let b = 17;
        
        // Split secrets
        let a_shares = SecretSharing::split_secret(a, 3);
        let b_shares = SecretSharing::split_secret(b, 3);
        
        // Send shares to other parties
        for (_i, share) in a_shares.iter().chain(b_shares.iter()).enumerate() {
            if share.party_id != party_id {
                party.send_share(share.party_id, share).await?;
            } else {
                party.shares.insert(share.party_id, share.clone());
            }
        }
    }
    
    // Wait to receive shares
    sleep(Duration::from_secs(1)).await;
    party.receive_share().await?;
    party.receive_share().await?;
    
    // Perform local computation (addition)
    let a_share = party.get_share(party_id).unwrap();
    let b_share = party.shares.values().find(|s| s.party_id == party_id && s.value != a_share.value).unwrap();
    
    let result_share = Share {
        value: a_share.value + b_share.value,
        party_id,
    };
    
    // Send result share back to party 1
    if party_id != 1 {
        party.send_share(1, &result_share).await?;
    } else {
        party.shares.insert(party_id, result_share.clone());
        
        // Collect all result shares
        sleep(Duration::from_secs(1)).await;
        party.receive_share().await?;
        party.receive_share().await?;
        
        // Reconstruct the result
        let all_shares: Vec<Share> = party.shares.values().cloned().collect();
        let result = SecretSharing::reconstruct_secret(&all_shares);
        
        println!("Final computation result: {}", result);
    }
    
    Ok(())
}
mod secret_sharing;
mod party;

use anyhow::Result;
use party::Party;
use secret_sharing::{SecretSharing, Share};
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <party_id> <port> <peer1_addr> <peer2_addr> ...", args[0]);
        std::process::exit(1);
    }
    
    let party_id: usize = args[1].parse()?;
    let port: u16 = args[2].parse()?;
    let peers = args[3..].to_vec();
    
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
        for (i, share) in a_shares.iter().chain(b_shares.iter()).enumerate() {
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
use rand::Rng;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Share {
    pub value: i64,
    pub party_id: usize,
}

pub struct SecretSharing;

impl SecretSharing {
    /// Split a secret into n shares where any n-1 shares reveal no information
    pub fn split_secret(secret: i64, num_parties: usize) -> Vec<Share> {
        let mut rng = rand::thread_rng();
        let mut shares = Vec::with_capacity(num_parties);
        let mut sum = 0;
        
        // Generate random shares for n-1 parties
        for party_id in 1..num_parties {
            let share_value = rng.gen_range(-1000..1000);
            shares.push(Share {
                value: share_value,
                party_id,
            });
            sum += share_value;
        }
        
        // The last share completes the secret
        shares.push(Share {
            value: secret - sum,
            party_id: num_parties,
        });
        
        shares
    }
    
    /// Reconstruct the secret from shares
    pub fn reconstruct_secret(shares: &[Share]) -> i64 {
        shares.iter().map(|s| s.value).sum()
    }
    
    /// Add two sets of shares locally (for MPC addition)
    pub fn local_add(a: &[Share], b: &[Share]) -> Result<Vec<Share>> {
        if a.len() != b.len() {
            return Err(anyhow!("Share counts don't match"));
        }
        
        Ok(a.iter().zip(b.iter())
            .map(|(a_share, b_share)| Share {
                value: a_share.value + b_share.value,
                party_id: a_share.party_id,
            })
            .collect())
    }
}
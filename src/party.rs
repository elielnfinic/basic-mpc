use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::{from_str, to_string};
use anyhow::Result;
use std::collections::HashMap;
use crate::secret_sharing::Share;

pub struct Party {
    id: usize,
    pub shares: HashMap<usize, Share>, // party_id -> share
    peers: Vec<String>, // addresses of other parties
    listener: TcpListener,
}

impl Party {
    pub async fn new(id: usize, port: u16, peers: Vec<String>) -> Result<Self> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        Ok(Self {
            id,
            shares: HashMap::new(),
            peers,
            listener,
        })
    }
    
    pub async fn receive_share(&mut self) -> Result<()> {
        let (mut socket, _) = self.listener.accept().await?;
        let mut buf = vec![0; 1024];
        
        let n = socket.read(&mut buf).await?;
        let share: Share = from_str(&String::from_utf8_lossy(&buf[..n]))?;
        
        self.shares.insert(share.party_id, share);
        Ok(())
    }
    
    pub async fn send_share(&self, party_id: usize, share: &Share) -> Result<()> {
        let mut stream = TcpStream::connect(&self.peers[party_id - 1]).await?;
        let serialized = to_string(share)?;
        stream.write_all(serialized.as_bytes()).await?;
        Ok(())
    }
    
    pub fn get_share(&self, party_id: usize) -> Option<&Share> {
        self.shares.get(&party_id)
    }
    
    pub fn add_shares(&self, other: &Share) -> Share {
        let my_share = self.shares.get(&self.id).unwrap();
        Share {
            value: my_share.value + other.value,
            party_id: self.id,
        }
    }
}
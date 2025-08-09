use anyhow::Result;
use libp2p::{
    mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::secret_sharing::Share;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpcMessage {
    pub party_id: usize,
    pub share: Share,
}

#[derive(NetworkBehaviour)]
pub struct MpcBehaviour {
    pub mdns: mdns::tokio::Behaviour,
}

impl MpcBehaviour {
    pub fn new(local_peer_id: PeerId) -> Result<Self> {
        let mdns_config = mdns::Config::default();
        let mdns = mdns::tokio::Behaviour::new(mdns_config, local_peer_id)?;
        
        Ok(Self {
            mdns,
        })
    }
}

pub struct P2pNode {
    pub swarm: Swarm<MpcBehaviour>,
    pub party_id: usize,
    pub connected_peers: HashMap<PeerId, usize>, // PeerId -> party_id
    pub shares_tx: mpsc::UnboundedSender<Share>,
    pub shares_rx: mpsc::UnboundedReceiver<Share>,
}

impl P2pNode {
    pub async fn new(party_id: usize) -> Result<Self> {
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        
        info!("Local peer id: {local_peer_id}");
        
        let transport = tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();
        
        let behaviour = MpcBehaviour::new(local_peer_id)?;
        let swarm = Swarm::new(transport, behaviour, local_peer_id, libp2p::swarm::Config::with_tokio_executor());
        
        let (shares_tx, shares_rx) = mpsc::unbounded_channel();
        
        Ok(Self {
            swarm,
            party_id,
            connected_peers: HashMap::new(),
            shares_tx,
            shares_rx,
        })
    }
    
    pub async fn start_listening(&mut self, port: u16) -> Result<()> {
        let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", port).parse()?;
        self.swarm.listen_on(listen_addr.clone())?;
        info!("Listening on {listen_addr}");
        Ok(())
    }
    
    pub async fn send_share(&mut self, _target_party_id: usize, share: &Share) -> Result<()> {
        // For now, just broadcast the share to all connected peers
        // In a real implementation, we'd send to specific peers
        debug!("Broadcasting share: {}", share.value);
        if let Err(e) = self.shares_tx.send(share.clone()) {
            warn!("Failed to send share: {e}");
        }
        Ok(())
    }
    
    pub async fn handle_swarm_event(&mut self, event: SwarmEvent<MpcBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(MpcBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    info!("Discovered peer: {peer_id} at {multiaddr}");
                    // Attempt to dial the discovered peer
                    if let Err(e) = self.swarm.dial(multiaddr) {
                        warn!("Failed to dial peer {peer_id}: {e}");
                    }
                }
            }
            SwarmEvent::Behaviour(MpcBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                for (peer_id, _) in list {
                    debug!("mDNS peer expired: {peer_id}");
                }
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to peer: {peer_id}");
                // For now, we'll assign party IDs in order of connection
                // In a real implementation, this would be negotiated
                let party_id = self.connected_peers.len() + 2; // Start from 2, assume we are party 1
                self.connected_peers.insert(peer_id, party_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnected from peer: {peer_id}");
                self.connected_peers.remove(&peer_id);
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {address}");
            }
            _ => {}
        }
    }
    
    pub fn get_connected_party_count(&self) -> usize {
        self.connected_peers.len()
    }
}

// Re-export for compatibility
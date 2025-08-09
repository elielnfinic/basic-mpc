# basic-mpc

Basic multiparty computation with p2p peer discovery.

## Features

- Basic additive secret key sharing
- Parties perform addition on their shares  
- Shares are recombined to reveal the result
- **NEW: P2P peer discovery using libp2p**
- Automatic peer discovery on local network using mDNS
- Backward compatibility with TCP-based networking

## Usage

### P2P Mode (Automatic Peer Discovery)

Run without specifying peer addresses to enable p2p discovery:

```bash
# Start party 1
cargo run -- 1 8000

# Start party 2 in another terminal
cargo run -- 2 8001

# Start party 3 in another terminal  
cargo run -- 3 8002
```

The parties will automatically discover each other on the local network using mDNS.

### Legacy TCP Mode

Run with peer addresses for traditional TCP networking:

```bash
# Party 1 (coordinator)
cargo run -- 1 8000 127.0.0.1:8001 127.0.0.1:8002

# Party 2
cargo run -- 2 8001 127.0.0.1:8000 127.0.0.1:8002

# Party 3
cargo run -- 3 8002 127.0.0.1:8000 127.0.0.1:8001
```

## Architecture

- `src/main.rs` - Entry point with dual-mode support
- `src/party.rs` - Legacy TCP-based party communication
- `src/p2p.rs` - **NEW: libp2p-based peer discovery and networking**
- `src/secret_sharing.rs` - Secret sharing algorithms

## Implementation Details

The p2p implementation uses:
- **mDNS** for automatic peer discovery on the local network
- **Noise protocol** for secure communication
- **Yamux** for multiplexing connections
- **TCP transport** for reliable communication

When running in p2p mode, parties automatically discover each other and establish secure connections without requiring manual configuration of peer addresses.

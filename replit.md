# Universal Connectivity - libp2p Chat Application

## Overview

This is a **Universal Connectivity** project showcasing libp2p's peer-to-peer communication capabilities. The project demonstrates real-time decentralized chat functionality across multiple programming languages and runtimes.

**Purpose**: Demonstrate libp2p's superpowers in establishing ubiquitous peer-to-peer connectivity using modern programming languages (Go, Rust, TypeScript) and various transport protocols.

**Current State**: Successfully configured and running in Replit environment with the Next.js frontend serving as the primary interface.

## Project Architecture

### Frontend (Primary)
- **Technology**: Next.js with TypeScript
- **Location**: `js-peer/` directory  
- **Port**: 5000 (configured for Replit)
- **Features**: Browser-based chat peer with WebTransport, WebRTC, and WebRTC-direct support

### Backend Implementations
- **Go Peer**: `go-peer/` - Chat peer with QUIC, TCP, WebTransport, WebRTC-direct support
- **Rust Peer**: `rust-peer/` - Chat peer with QUIC, WebRTC-direct support

### Transport Protocols Supported
- WebTransport ✅ (JS, Go)
- WebRTC ✅ (JS)  
- WebRTC-direct ✅ (JS, Go, Rust)
- QUIC ✅ (Go, Rust)
- TCP ✅ (Go)

## Recent Changes (September 10, 2025)

### Replit Environment Setup
- ✅ Installed Node.js 20, Go 1.24, and Rust stable toolchains
- ✅ Configured Next.js to bind to `0.0.0.0:5000` for Replit proxy compatibility
- ✅ Removed static export configuration to enable proper server-side rendering
- ✅ Updated development and production scripts for correct host/port binding
- ✅ Set up automated workflow for frontend development server
- ✅ Configured deployment for production with autoscale target

### Configuration Changes
- **next.config.js**: Removed `output: 'export'`, added Replit-specific configurations
- **package.json**: Updated dev/start scripts to use `--hostname 0.0.0.0 --port 5000`
- **Workflow**: Frontend development server configured to run automatically on port 5000

## Key Dependencies

### Frontend (js-peer)
- Next.js 14.2.25
- libp2p ecosystem packages (@libp2p/webrtc, @libp2p/webtransport, etc.)
- React 18.3.1
- Tailwind CSS for styling
- TypeScript for type safety

### Protocol & Communication
- GossipSub for decentralized messaging
- WebRTC and WebTransport for browser connectivity
- Multiple libp2p transport protocols

## Development Workflow

### Primary Development (Frontend)
```bash
cd js-peer
npm run dev  # Runs on 0.0.0.0:5000
```

### Backend Peers (Optional)
```bash
# Go peer
cd go-peer && go run .

# Rust peer  
cd rust-peer && cargo run
```

## User Preferences

- **Primary Focus**: Frontend Next.js application (js-peer)
- **Environment**: Replit cloud development
- **Deployment**: Autoscale for stateless web application
- **Port**: 5000 (Replit standard)

## Project Status

✅ **Ready for Development**: The project is fully configured and running in the Replit environment. The Next.js frontend is accessible at the provided preview URL and successfully initializes libp2p peer connections.

## Notes

- The application uses experimental HTTPS in the original configuration, but this has been adapted for Replit's proxy environment
- Bootstrap nodes are automatically connected for peer discovery
- The application supports real-time peer-to-peer chat across multiple transport protocols
- All three language implementations (JS, Go, Rust) can interoperate in the same network
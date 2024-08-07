# Simple Blockchain in Rust

This project is a simple implementation of a blockchain written in Rust. Actually, this project is a Rust rewrite of the [Naivechain](https://github.com/lhartikk/naivechain) project. It includes basic functionalities such as creating a blockchain, adding blocks, and a peer-to-peer network for sharing blocks between nodes. This implementation does not include a consensus protocol.

## Project Structure

The project is divided into several modules:

- **`main.rs`**: The entry point of the application. It sets up the HTTP server and the peer-to-peer network.
- **`api.rs`**: Handles API requests for interacting with the blockchain, including retrieving and mining blocks.
- **`engine.rs`**: Manages the blockchain's internal logic and communication between nodes.
- **`chain.rs`**: Defines the `Block` and `Chain` structures and implements the logic for creating and validating blocks.
- **`net.rs`**: Configures the peer-to-peer network using `libp2p` and manages message transmission between nodes.

## Modules Overview

### `main.rs`

- Uses the `actix_web` framework to set up an HTTP server for handling API requests.
- Parses command-line arguments using `clap`.
- Initializes the blockchain with a genesis block.
- Configures the network and starts the engine to handle blockchain operations.

### `api.rs`

- Defines the `ApiState` struct, which holds the blockchain and transmission handlers.
- Provides API endpoints for:
  - Retrieving the current blockchain (`/blocks/get`).
  - Mining a new block (`/blocks/mine`).
  - Viewing peers (`/peers`).
  - Adding new peers (`/addpeer`).

### `engine.rs`

- Manages the blockchain logic, including handling incoming P2P messages.
- Ensures the local blockchain is up-to-date by responding to and sending messages across the network.

### `chain.rs`

- Defines the `Block` struct with fields like `index`, `previous_hash`, `timestamp`, `data`, and `hash`.
- Implements methods for creating and validating blocks, as well as managing the blockchain's state.

### `net.rs`

- Configures the P2P network using `libp2p`, enabling nodes to discover each other via mDNS and communicate using the GossipSub protocol.
- Manages incoming and outgoing P2P messages, ensuring blocks are shared across nodes.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (Rust package manager)

### Running the Application

1. Clone the repository:
```sh
$ git clone <repository-url>
$ cd naivechain-rs
```
2. Build the project:
```sh
$ cargo build
```
3. Run the application:
```sh
$ cargo run -- --port <PORT_NUMBER>
```
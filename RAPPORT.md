# README

## Introduction

This project is a simplified educational implementation of a blockchain inspired by the Bitcoin protocol. It was developed as part of a course on distributed systems and cryptocurrencies, with the goal of exploring the core mechanisms of blockchain technology through practical application — including block structure, proof of work (PoW), cryptographic hashing, block tree construction, and mining.

The main objective was to build a functional blockchain that respects several technical constraints:

- each block must generate its own unique hash;
- include a unique nonce;
- and carry a miner name different from "changemeyoufool".

Rather than aiming for the full complexity of a real-world cryptocurrency, the focus was placed on understanding fundamental principles through a clear, modular, and testable implementation. This project also strengthened our skills in Rust programming, distributed algorithm design, and network communication via a REST server.

## Project structure

The codebase is divided into clearly separated modules:

- **block.rs** — defines the structure of a block and implements the proof-of-work mechanism.
- **miner.rs** — contains the miner logic: receiving blocks, building the local blockchain, mining new blocks, and sending them to the network.
- **network.rs** — handles communication with the backend server using HTTP (GET and POST requests).
- **simpletree.rs** — generic tree structure used to represent the blockchain with its possible forks.
- **server/main.rs** — central server that stores blocks in memory and validates their proof-of-work.

## Module description

### `block.rs`
- `Block` and `DanceMove` correspond to the structure given in the project statement
  (parent_hash, miner, dancemove, nonce).
- `hash_block` computes the SHA-256 hash over the fields in top-down order, with
  integers (`dancemove` as u8, `nonce` as u64) serialized in little-endian.
- `pow_check` checks that the first `difficulty` bits of the hash are 0. The function
  iterates over the full zero bytes first, then applies a mask on the high bits of
  the next byte for the remaining bits.
- `solve_block` is the nonce search loop: it computes the hash and picks a new random
  nonce until the PoW is satisfied (or `max_iteration` is reached).
- `is_block_valid` rejects the names "changemeyoufool" and "Genesis", then checks the PoW.
- `is_genesis` checks that the miner is "Genesis", the parent_hash is empty, and the PoW is valid.

### `simpletree.rs`
Generic tree (`TreeNode<T>`) with a `Parenting` trait used to find a node by its
identifier. Main methods: `new`, `insert`, `look_for_parent` (recursive search),
`depth`, `remove`. This is used to represent the blockchain and its forks.

### `miner.rs`
- `Blockchain::new_from_genesis_and_vec` rebuilds the tree from a list of blocks by
  looking for each block's parent in the tree. Since the reception order is not
  guaranteed, several passes are done until no new block can be placed. Remaining
  blocks are returned as orphans.
- `longest_chain` recursively finds the longest chain starting from a node.
- `mine` is the main mining loop:
  1. Starts a network thread that synchronizes with the server.
  2. Looks for a genesis block. If none exists, creates and sends one.
  3. Gets the latest blocks, rebuilds the tree, picks the longest chain, mines a
     new block on top, and sends it to the server.
- `print_chain` displays the current blockchain state retrieved from the server.

### `network.rs`
Client-server communication using `reqwest` (blocking mode):
- `get_blocks` (GET `/blocks`) returns the list of all blocks on the server.
- `send_block` (POST `/postblock`) sends a mined block to the server.
- `NetworkConnector::sync` runs in a dedicated thread: it periodically fetches the
  server state, forwards it to the miner thread through an mpsc channel, and pushes
  the miner's newly mined blocks to the server.

### `server/main.rs`
HTTP server based on `rouille` with two routes:
- `GET /blocks` returns the stored blocks as JSON.
- `POST /postblock` validates the PoW and inserts the block into an in-memory
  `Mutex<HashMap<...>>`.

## Use of AI

In accordance with the course requirements, each function in the code is annotated
with `// Programmed without AI.` or `// Programmed with AI assistance.` to indicate
whether AI was used.

**Done without AI assistance:**
- Block and DanceMove data structures, constructors, getters.
- Basic validations (`is_block_valid`, `is_genesis`).
- Command-line argument parsing with `clap`.
- `find_genesis`, `create_genesis`, the main dispatch.
- The tree's trivial methods (`new`, `insert`, getters).

**Done with AI assistance:**
- Bit-level proof-of-work check (`pow_check`).
- SHA-256 hashing with correct field order and little-endian encoding (`hash_block`).
- Nonce search loop (`solve_block`).
- Rebuilding the tree from an unordered list of blocks (`new_from_genesis_and_vec`).
- Recursive longest-chain search (`longest_chain`).
- Tree formatting with box-drawing characters (`print_tree`).
- HTTP client (reqwest) and `NetworkConnector` thread.
- HTTP server with `rouille`, JSON routing, and concurrency handling.
- Unit and integration tests.

## Personal note

I joined the course relatively late and had to catch up on part of the material
while working on the project. Some technical aspects (notably bit-level hashing,
threads with `mpsc`, and HTTP libraries) required external help, which I made
sure to understand before integrating it. The overall project structure and the
simpler parts were done by myself.

## How to run

```
# Build
cargo build

# Start the server (default port 8080)
cargo run --bin server

# Start a miner
cargo run --bin miner -- mine

# Print the current blockchain
cargo run --bin miner -- print
```
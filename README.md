# VaporDB

A blazing-fast, lightweight key-value store inspired by Redis, built with a Rust backend.

## Overview

VaporDB is a modern key-value database designed for simplicity, speed, and developer productivity. All core data logic is implemented in Rust, ensuring high performance and reliability. The SvelteKit frontend provides an intuitive web interface for interacting with and visualizing your data. VaporDB is ideal for rapid prototyping, learning, and as a lightweight alternative to heavier solutions.

## Features

- Redis-like key-value operations: `SET`, `GET`, `DELETE`, and more
- In-memory storage with optional persistence (configurable)
- Simple RESTful API powered by Rust
- Web UI built with SvelteKit for easy data management
- Optional TTL (time-to-live) for expiring keys
- Easy to deploy and cross-platform

## Getting Started

**Prerequisites**

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (for frontend)

**Clone the repository**

```bash
git clone https://github.com/tejxsmore/vapordb.git
cd vapordb
```

**Backend: Build & Run**

```bash
cd backend
cargo build --release
cargo run
```

The server will start (default: `localhost:8080`).

**Frontend: Start Dev Server**

```bash
cd frontend
npm install
npm run dev
```

Visit `http://localhost:5173` to access the UI.

## Usage

You can interact with VaporDB via:

- The SvelteKit web UI
- REST API endpoints (e.g., `/set`, `/get`, `/delete`)

### Example API Usage

```http
POST /set
{
  "key": "username",
  "value": "alice",
  "ttl": 60
}
```

```http
GET /get?key=username
```

## Contributing

Contributions are welcome! Please open issues or submit pull requests for new features, bug fixes, or improvements.

## License

MIT License

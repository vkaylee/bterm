# BTerminal

**BTerminal** is a high-performance, multi-session Web Terminal application built with Rust and Xterm.js.

## Overview
BTerminal allows you to manage multiple terminal sessions through a clean web interface. It features real-time synchronization across clients, session persistence, and is delivered as a single portable binary.

## Documentation
Comprehensive documentation is available in the [docs/](./docs/) directory:

- [System Architecture](./docs/system-architecture.md)
- [API Reference](./docs/api-reference.md)
- [Code Standards](./docs/code-standards.md)
- [Changelog](./CHANGELOG.md)

## Quick Start
```bash
# Run the application
cargo run

# Access via browser
# http://localhost:3000
```

## Features
- **Multi-session:** Create and manage independent terminal sessions.
- **Shared View:** Multiple clients can join the same session.
- **Responsive:** Optimized for both desktop and mobile use.
- **Zero Dependencies:** All frontend assets are embedded in the binary.

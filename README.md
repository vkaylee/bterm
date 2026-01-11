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
- **Modern Dashboard:** Content-first layout with real-time session monitoring and count.
- **Pro Header:** Quick access to session IDs, connectivity status, and easy exit.
- **Mobile Optimized:** 2-row virtual keyboard, Sticky Modifiers (Ctrl/Alt), and visual viewport awareness.
- **Shared View:** Multiple clients can join the same session.
- **Zero Dependencies:** All frontend assets are embedded in the binary.

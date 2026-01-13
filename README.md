# BTerminal

**BTerminal** is a high-performance, multi-session Web Terminal application built with Rust and Xterm.js.

## Overview
BTerminal allows you to manage multiple terminal sessions through a clean web interface. It features real-time synchronization across clients, session persistence, and is delivered as a single portable binary.

## Documentation
Comprehensive documentation is available in the [docs/](./docs/) directory:

- [System Architecture](./docs/system-architecture.md)
- [API Reference](./docs/api-reference.md)
- [Deployment Guide](./docs/deployment-guide.md)
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
- **Built-in Authentication:** Secure access control using SQLite, Argon2id hashing, and session cookies.
- **Multi-session:** Create and manage independent terminal sessions.
- **Modern Dashboard:** Content-first layout with real-time session monitoring and count.
- **Pro Header:** Quick access to session IDs, connectivity status, and easy exit.
- **Mobile Optimized:** 2-row virtual keyboard, Sticky Modifiers (Ctrl/Alt), and visual viewport awareness.
- **Touch Selection:** Native-like text selection on mobile devices via long-press.
- **Smart Clipboard:** Context-aware Copy/Paste (Ctrl+C/V) and seamless integration with system clipboard.
- **High Performance:** Automatic 3-tier rendering engine (WebGL -> Canvas -> DOM) for 60fps performance.
- **Robust Cleanup:** Process Group (PGID) management ensures no orphaned background processes.
- **Shared View:** Multiple clients can join the same session.
- **Zero Dependencies:** All frontend assets are embedded in the binary.

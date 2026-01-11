# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Dynamic Port Selection**: The server now checks the `PORT` environment variable, defaults to `3000`, and automatically falls back to an available random port if the preferred ports are occupied.
- **Port Integration Tests**: New test suite in `tests/port_integration.rs` to verify binding logic and environment variable overrides.
- **Dynamic E2E Isolation**: Playwright now uses worker-scoped fixtures to spawn independent backend instances on random ports, ensuring zero state leakage during parallel testing.

### Changed
- Updated `src/main.rs` to use `tokio::net::TcpListener` with a fallback loop instead of a hardcoded address.
- Refactored E2E tests to use a custom `test` fixture with dynamic `baseURL` instead of a global `webServer`.

## [0.1.0] - 2026-01-11



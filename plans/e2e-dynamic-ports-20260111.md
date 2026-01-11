# Implementation Plan: Dynamic E2E Port Isolation

## Approach
- **Why**: Current Playwright config uses a shared server on port 3000, preventing true parallelism and causing state leakage.
- **Solution**: Use Playwright worker-scoped fixtures to start a unique backend instance per worker using the `PORT` environment variable.

## Steps
1. **Remove global webServer**: Disable the global `webServer` in `playwright.config.ts`.
2. **Create Worker Fixture**: Implement a fixture in a new file `e2e/fixtures.ts` that:
   - Spawns `cargo run` with a dynamic port.
   - Parses the stdout to find the bound URL.
   - Provides this URL to tests.
   - Cleans up the process on worker teardown.
3. **Refactor Tests**: Update `e2e/tests/*.spec.ts` to use the custom fixture.
4. **Verification**: Run tests with high worker count (`--workers=3`).

## Timeline
- Implementation: 30 min
- Refactoring: 15 min
- Testing: 15 min
Total: 1 hour

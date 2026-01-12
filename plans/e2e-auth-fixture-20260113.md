# Implementation Plan: E2E Auth Fixture

## Approach
To ensure all E2E tests pass after the authentication system was introduced, we will update the Playwright fixtures to automatically perform a login for every test worker.

### Why this solution?
- **Real Login**: Verifies the actual auth flow instead of bypassing it.
- **Worker Isolation**: Each test worker starts its own backend with an in-memory database to prevent state leakage and file lock conflicts.
- **Automatic**: Developers don't need to manually write `login` in every `.spec.ts` file.

## Steps

1. **Modify Backend Spawn in Fixture** (5 min)
   - Set `DATABASE_URL: 'sqlite::memory:'` in `e2e/fixtures.ts` to ensure each worker has a clean, isolated database.

2. **Implement Global Auth Fixture** (15 min)
   - Update `e2e/fixtures.ts` to include a `storageState` or a `beforeEach` logic that:
     1. Calls `POST /api/auth/login` with `admin/admin`.
     2. Extracts the `set-cookie` header.
     3. Injects the cookie into the browser context.

3. **Verify Existing Tests** (10 min)
   - Run a few representative tests (e.g., `terminal-interaction.spec.ts`) to ensure they can now bypass the 401 screen.

4. **Add Dedicated Auth E2E Test** (10 min)
   - Create `e2e/tests/auth.spec.ts` to specifically test the Login UI and unauthorized access.

## Timeline
| Phase | Duration |
|-------|----------|
| Backend Spawn Config | 5 min |
| Auth Fixture Logic | 15 min |
| Verification | 10 min |
| New Auth Tests | 10 min |
| **Total** | **40 min** |

## Rollback Plan
- Revert changes to `e2e/fixtures.ts`.
- If backend changes were made (though not planned here), revert them using `kit_restore_checkpoint`.

## Security Checklist
- [x] Use `admin/admin` only for E2E tests.
- [x] Ensure `DATABASE_URL` isolation to prevent cross-test interference.
- [x] Verify that cookies are correctly handled (HttpOnly).

## Next Steps
```bash
/cook @plans/e2e-auth-fixture-20260113.md
```

# Implementation Plan: Subprocess Group Management & Cleanup

### Approach
- **Process Groups:** Use `setsid()` (already provided by `portable_pty`) to ensure the shell is a session leader and process group leader.
- **Manual Cleanup:** Update `PtyManager` to store the child process handle. Implement a `shutdown()` method that sends `SIGKILL` to the process group (`-PID`).
- **Safety Net:** Use `nix` crate to manage signals reliably.
- **Integration:** Ensure `Session` calls `pty_manager.shutdown()` when dropped or explicitly removed.

### Steps

1. **Install Dependencies**
   - Add `nix` to `Cargo.toml`.
   - Add `libc` for Linux-specific `prctl` if we find a way to inject it, otherwise focus on PGID kill.

2. **Update `PtyManager` Structure**
   - Files: `src/pty_manager.rs`
   - Add `child: Arc<Mutex<Option<Box<dyn Child + Send>>>>` to `PtyManager`.
   - Modify `new()` to store the child handle instead of just waiting in a detached thread.

3. **Implement Shutdown Logic**
   - Files: `src/pty_manager.rs`
   - Implement `fn shutdown(&self)`:
     - Get PID from child.
     - Use `nix::sys::signal::kill(Pid::from_raw(-pid), Signal::SIGKILL)`.
   - Implement `Drop` for `PtyManager` to call `shutdown()`.

4. **Update Session Lifecycle**
   - Files: `src/session.rs`
   - Ensure `remove_session` or the monitor loop triggers `pty_manager.shutdown()`.

5. **Verification**
   - Create a test case that spawns a long-running process (e.g., `sleep 100`) and verifies it's killed when the session is removed.

### Timeline
| Phase | Duration |
|-------|----------|
| Dependencies | 5 min |
| PtyManager Update | 15 min |
| Shutdown Impl | 10 min |
| Session Integration | 10 min |
| Testing | 20 min |
| **Total** | **1 hour** |

### Rollback Plan
- Revert changes to `src/pty_manager.rs` and `src/session.rs`.
- Remove `nix` dependency.

### Security Checklist
- [x] Process cleanup to prevent resource exhaustion.
- [ ] Ensure only the specific process group is killed.

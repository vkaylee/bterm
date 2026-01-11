# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **API:** Added `DELETE /api/sessions/:id` endpoint to terminate sessions and close PTY processes.
- **E2E Testing:** Integrated Playwright for automated testing of session management and terminal interactions.
- **UI:** Added on-screen control buttons for mobile users (Ctrl, Alt, Tab, Arrows).

### Fixed
- **Backend:** Resolved a critical lifetime error in `pty_manager.rs` where the shell reference was escaping the constructor.
- **Frontend:** Fixed a JavaScript syntax error in `index.html` (removed TypeScript-style casting).
- **Mobile Rendering:** Fixed a "black screen" issue on mobile by forcing terminal focus and refresh cycles.
- **WebSocket:** Improved WebSocket connection stability and error handling.

### Changed
- **UI:** Increased default font size to 16px for better readability on mobile devices.
- **E2E:** Switched to unique session naming and added dialog handlers to prevent test hangs.

# Poco Voxel Sandbox — Dev10 / Dev11 reconstruction status

This is a reconstruction based on the uploaded Dev7 source. It is a working source candidate, **not yet a verified Android build**.

## Present in this source candidate

### Dev10 gameplay
- First-person camera with mouse-drag / right-thumb look.
- Keyboard and thumbstick movement, sprint, jump, gravity and ground detection.
- Horizontal collision, ceiling checks, and a one-block step-up attempt while grounded.
- Creative flight retained with keyboard and touch UP/DOWN controls.
- Voxel raycast, block selection, breaking and placement.
- Placement is rejected when the target block would intersect the player's body.
- Chunk-boundary neighbour remeshing retained from the baseline.
- Performance overlay and touch HUD.

### Dev11 world reliability
- Manual SAVE / LOAD controls and F5 / F9 desktop shortcuts.
- Versioned JSON save format (version 2) with stable block IDs.
- FNV-1a checksum includes seed, chunk count, coordinates, block counts and block IDs.
- Validation rejects unsupported versions, oversized saves, invalid block IDs, malformed chunk sizes and duplicate chunk coordinates.
- Save writes are staged to a temporary file and synced before replacement.
- Previous valid primary save is staged as a backup; corrupt primary data does not overwrite a known-good backup.
- Load falls back to the backup if the primary save is missing or invalid.
- Background saving remains disabled; saving is manual and synchronous.

### Android build pipeline
- GitHub Actions installs stable Rust, the Android ARM64 target, Android SDK platform 35 and NDK 27.2.12479018.
- CI runs Rust unit tests before packaging.
- Release APK is uploaded as a downloadable workflow artifact.
- Rust build cache and workflow concurrency controls are enabled.

## Not yet verified / remaining work

- This environment has no Rust/Cargo toolchain, so no local `cargo test`, `cargo check`, or APK compilation has been performed.
- The GitHub Actions workflow has not yet been run against the user's repository.
- Android's actual writable save directory and persistence across app restarts still require device verification.
- Current prototype renders a fixed 3×3 chunk area. Continuous chunk streaming, large-world traversal and distant terrain LOD are not implemented in this milestone.
- Cave passage/headroom, chunk-edge edits, collision edge cases and interrupted-save recovery need CI/device testing beyond the included unit tests.
- No POCO F6 install or runtime test has been performed.

## Controls

- Move: WASD / left touchscreen joystick
- Look: mouse drag / right touchscreen area
- Jump: Space / touchscreen JUMP
- Sprint: Left Shift / touchscreen RUN
- Flight toggle: F / FLY; UP/DOWN buttons while flying
- Break/place: Q/E, right/middle mouse, or touchscreen BREAK/PLACE
- Select block: 1–5 or touchscreen material bar
- Save/load: F5/F9 or touchscreen SAVE/LOAD
- Help/performance: H / F3

The build is only considered complete after GitHub Actions produces an APK artifact successfully. Device compatibility is only confirmed after installing and running it on the POCO F6.

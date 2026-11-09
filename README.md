# Poco Voxel Sandbox — Android test environment

This repository is the home for the Rust + Macroquad Android ARM64 test build.

## Important: source upload is still required

The Dev7 source and the Dev10/Dev11 reconstruction candidate currently exist as ZIP files in the ChatGPT conversation. The GitHub connector can create and edit text files, but it cannot transfer that ZIP archive into this repository. **Do not run the Android workflow until the actual project files are uploaded to the repository root.**

Upload the *contents* of the project folder (not the outer ZIP and not an extra nested project folder) so that `Cargo.toml`, `src/`, and `assets/` are at the repository root.

## Build

Once the Rust project files are present:

1. Open **Actions**.
2. Select **Build POCO F6 Android APK**.
3. Choose **Run workflow**.
4. Wait for the job to finish.
5. Download the `poco-voxel-sandbox-arm64-apk` artifact from the successful run.

The workflow runs Rust unit tests before attempting the Android APK build.

## Version plan

- **V1 — Main game:** original build on `main`; keep it unchanged.
- **V2 — Tree-generation fix:** this branch is the V1-focused test build. It adjusts the starting biome conditions to make the spawn area tree-friendly and checks generated tree blocks. Its Android application ID is separate so it can coexist with V1.
- **V3 — Biome Testing:** a later, separate experimental build with its own 3D test world, biome selection and regeneration controls. V3 is not implemented yet.

The fixed **3×3 chunk world is intentional**. It is a controlled test environment, not an unfinished endless-world implementation.

## Current status

V2 Rust unit tests pass. The Android build is being retried after correcting the NDK environment variable and assigning V2 a distinct Android application ID. No APK is ready until the Android build succeeds and the artifact is available. POCO F6 runtime behavior still needs device testing.

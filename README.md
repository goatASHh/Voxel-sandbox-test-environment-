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

## Test scope

The fixed **3×3 chunk world is intentional**. It is a controlled test environment for movement, voxel editing, chunk seams, caves, persistence and performance—not an unfinished endless-world implementation.

## Current status

The Dev10/Dev11 source candidate is not yet compiler-verified. No APK is claimed until GitHub Actions completes successfully. POCO F6 runtime compatibility still requires a device test after the APK is built.

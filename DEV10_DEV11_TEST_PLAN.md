# Dev10 / Dev11 verification checklist

## Automated checks (GitHub Actions)

- Rust unit tests compile and run before Android packaging.
- Save/load round trip preserves the seed and edited voxel data.
- Corrupt primary save recovers the last known-good backup.
- Duplicate chunk coordinates are rejected.
- Player placement guard rejects blocks intersecting the player's AABB.

## Required gameplay checks after an APK exists

1. Walk into a flat wall from each horizontal direction; the player must not pass through it.
2. Walk up a one-block ledge while grounded; the step-up should work without allowing wall climbing.
3. Jump into a low ceiling; upward movement should stop without clipping through it.
4. Walk off a ledge and land; the player should settle on the surface without jitter.
5. Try to place a block in the player's occupied space; placement should be rejected.
6. Break and place blocks on each edge of the currently loaded chunk area; neighbouring visible faces should update.
7. Save, change several blocks, then load; the saved edits should return.
8. Save twice, corrupt/remove the primary save in a test build, and verify backup recovery.
9. Force-close and relaunch the app; confirm the save remains accessible in Android app storage.
10. Keep movement and look touches active simultaneously; both controls should remain responsive.

## Known scope limits

- The current demo world is a fixed 3×3 chunk patch, not an endless streamed world.
- No claim is made about Android persistence or POCO F6 performance until tested on hardware.

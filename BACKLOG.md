# Asteroids Rust Port - Backlog

## Open Tasks

- [ ] [gameplay] chunks_explo spawn one explosion per frame — damage output is framerate-bound. Consider dt-scaling or fixed-rate accumulator for framerate independence. (severity: minor, 2026-03-27)
- [ ] [visual] Visual testing of Phase 1 rendering changes — validate tonemapping variants (faithful, spectral bleed, ACES, Reinhard), SDF quality, MSAA 4x performance. (severity: minor, 2026-03-28)
- [ ] [visual] Visual testing of gamepad + world-space controls — verify twin-stick feel, aim smoothing, cone teleport targeting, engine fire direction (severity: minor, 2026-03-30)
- [ ] [visual] Engine fire not visibly ejected at high ship speeds — backward kick scales with ship_speed but still doesn't look right. Consider pure ship-relative visual approach or larger base kick. (severity: minor, 2026-03-28)
- [ ] [ui/i18n] Prepare for internationalization: extract all player-visible strings (pause menu labels, HUD text, tooltips, weapon names, calibration menu) into a centralized string table (e.g. `src/strings.rs` or `src/locale.rs`). No translation yet — just indirection so display text is not hardcoded inline. (severity: minor, 2026-03-28)

## Completed Tasks

- [x] Gamepad + world-space controls — full implementation (completed: 2026-03-29)
- [x] Task 13: Config restructure (split Globals into sub-structs)
- [x] Task 14: Bug fixes (raw pointers, dead code, EntityKind, dedup)
- [x] Engine integration tests (camera, spawn, game state)
- [x] Game manual (docs/GAME_MANUAL.md)
- [x] README update for V2 Phase 0 progress

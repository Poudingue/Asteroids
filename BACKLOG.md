# Asteroids Rust Port - Backlog

## Open Tasks

- [ ] [visual] Visual testing of Phase 1 rendering changes — validate tonemapping variants (faithful, spectral bleed, ACES, Reinhard), SDF quality, MSAA 4x performance. (severity: minor, 2026-03-28)
- [ ] [visual] Visual testing of gamepad + world-space controls — verify twin-stick feel, aim smoothing, cone teleport targeting, engine fire direction (severity: minor, 2026-03-30)
- [ ] [visual] Engine fire not visibly ejected at high ship speeds — backward kick scales with ship_speed but still doesn't look right. Consider pure ship-relative visual approach or larger base kick. (severity: minor, 2026-03-28)
- [ ] [i18n] Internationalization system — locale files (RON), system locale detection, string extraction from HUD/pause menu. Spec: docs/superpowers/specs/2026-04-03-i18n-locale-glyph-design.md (severity: minor, 2026-04-03)
- [ ] [rendering] Extended glyph system — 26 hand-designed lowercase glyphs, 5 accent marks, ~15 composed French accented chars, three-tier lookup with override support. Spec: docs/superpowers/specs/2026-04-03-i18n-locale-glyph-design.md (severity: minor, 2026-04-03)
- [ ] [i18n] Western European character support (scope B) — German ß/ä/ö/ü, Spanish ¿/¡, Scandinavian å/æ/ø, Portuguese ã/õ. ~30 additional glyph definitions. (severity: minor, 2026-04-03)
- [ ] [i18n] Full Latin Extended character support (scope C) — Polish, Czech, Hungarian etc. 50+ additional glyphs. (severity: minor, 2026-04-03)
- [ ] [architecture] Distortion field system — analytical field evaluation replacing one-shot shockwave, expanding ring shockwaves, gravity wells, time dilation via proper_time. Spec: docs/superpowers/specs/2026-04-01-distortion-field-design.md (severity: major, 2026-04-03)
- [ ] [rendering] MSDF text rendering — replace polygon-based glyph system with MSDF for better quality and scalability. Future replacement for current system. (severity: minor, 2026-04-03)

## Completed Tasks

- [x] Gamepad + world-space controls — full implementation (completed: 2026-03-29)
- [x] Task 13: Config restructure (split Globals into sub-structs)
- [x] Task 14: Bug fixes (raw pointers, dead code, EntityKind, dedup)
- [x] Engine integration tests (camera, spawn, game state)
- [x] Game manual (docs/GAME_MANUAL.md)
- [x] README update for V2 Phase 0 progress

# Asteroids Rust Port - Backlog

## Open Tasks

- [ ] [gameplay] chunks_explo spawn one explosion per frame — damage output is framerate-bound. Consider dt-scaling or fixed-rate accumulator for framerate independence. (severity: minor, 2026-03-27)
- [ ] [physics] Cap dt to prevent physics explosions on frame stalls (alt-tab, window drag). Max dt ~0.05s (20fps floor). (severity: minor, 2026-03-27)
- [ ] [visual] HDR output support — switch surface from Bgra8Unorm to Rgba16Float, replace clamp(0..255)+redirect_spectre_wide with proper tonemapping (ACES/Reinhard), define paper white (80 nits) and let flashes exceed naturally. Current pipeline is already linear so architecture is well-suited. (severity: minor, 2026-03-28)
- [ ] [visual] Engine fire not visibly ejected at high ship speeds — backward kick scales with ship_speed but still doesn't look right. Consider pure ship-relative visual approach or larger base kick. (severity: minor, 2026-03-28)
- [ ] [ui/i18n] Prepare for internationalization: extract all player-visible strings (pause menu labels, HUD text, tooltips, weapon names, calibration menu) into a centralized string table (e.g. `src/strings.rs` or `src/locale.rs`). No translation yet — just indirection so display text is not hardcoded inline. (severity: minor, 2026-03-28)
- [ ] [rendering/Phase1] Remove jitter AA system — current_jitter_double, current_jitter_coll_table, and all jitter-related logic in rendering and collision. Obsolete with new multi-pass pipeline. (severity: minor, 2026-03-28)
- [ ] [rendering/Phase1] Add MSAA anti-aliasing for all render passes — world geometry (ship, asteroids, projectiles), HUD, and text. Toggleable from pause menu: off (1×), 2×, 4× via MSAA_SAMPLE_COUNT constant. Applies to interface/text as well, not just world geometry. Consider SMAA/FXAA post-process pass for remaining edges (particles). (severity: minor, 2026-03-28)

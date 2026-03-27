# Asteroids Rust Port - Backlog

## Open Tasks

- [ ] [gameplay] chunks_explo spawn one explosion per frame — damage output is framerate-bound. Consider dt-scaling or fixed-rate accumulator for framerate independence. (severity: minor, 2026-03-27)
- [ ] [physics] Cap dt to prevent physics explosions on frame stalls (alt-tab, window drag). Max dt ~0.05s (20fps floor). (severity: minor, 2026-03-27)
- [ ] [visual] HDR output support — switch surface from Bgra8Unorm to Rgba16Float, replace clamp(0..255)+redirect_spectre_wide with proper tonemapping (ACES/Reinhard), define paper white (80 nits) and let flashes exceed naturally. Current pipeline is already linear so architecture is well-suited. (severity: minor, 2026-03-28)
- [ ] [visual] Engine fire not visibly ejected at high ship speeds — backward kick scales with ship_speed but still doesn't look right. Consider pure ship-relative visual approach or larger base kick. (severity: minor, 2026-03-28)

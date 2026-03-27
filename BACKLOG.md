# Asteroids Rust Port - Backlog

## Open Tasks

- [ ] [gameplay] chunks_explo spawn one explosion per frame — damage output is framerate-bound. Consider dt-scaling or fixed-rate accumulator for framerate independence. (severity: minor, 2026-03-27)
- [ ] [physics] Cap dt to prevent physics explosions on frame stalls (alt-tab, window drag). Max dt ~0.05s (20fps floor). (severity: minor, 2026-03-27)

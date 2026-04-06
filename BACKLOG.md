# Asteroids — Backlog

## Testing & Capture Tools

- [ ] [testing] Screenshot tool — capture screenshots for visual comparison of rendering parameters across pre-recorded scenarios; supports automated A/B comparison workflows (severity: major, 2026-04-06)
- [ ] [testing] Video capture — record video output for visual comparison of rendering across scenarios; useful for regression testing visual quality (severity: major, 2026-04-06)
- [ ] [testing] Scenario recording UI — "Record Scenario" checkbox near "New Game" in pause menu; forces fixed-dt at screen refresh rate (e.g. 60fps → dt=1/60), records both user inputs and full gameplay state, produces a deterministically replayable scenario file. Dual purpose: visual rendering comparison AND engine determinism verification. Builds on Phase 0 fixed-dt + RON/zstd+bincode recording infrastructure. (severity: major, 2026-04-06)

---

## Visual Testing (unblocks other work)

- [ ] [visual] NEEDS VISUAL TESTING: All new rendering/UI features from 2026-04-04 — HDR pipeline, pause menu redesign, MSAA toggle, tonemap variants, slider drag fix (severity: major, 2026-04-04)
- [ ] [visual] Validate Phase 1 rendering: tonemapping variants, SDF quality, HDR, pause menu (severity: minor, 2026-04-04)
- [ ] [visual] Validate gamepad + world-space controls — twin-stick feel, aim smoothing, cone teleport, engine fire direction (severity: minor, 2026-03-30)

---

## Rendering Pipeline

- [ ] [rendering] Fix rendering layer order: background → stars → smoke → chunks → sparkles (light-trails from physical collisions, intensity dosed by collision force) → bullets → asteroids → explosions → ship (severity: major, 2026-04-06)
- [ ] [rendering] Mutualize stars/bullets/sparkles rendering — same visual principles (configurable capsule size, brightness falloff rate, color, "shutter speed" param defaulting to 100%). Shutter speed: 0% = pure circle (no streak), 100% = physically correct trail length, >100% exaggerates trail beyond physical. Time-exposure metaphor: lower = shorter trail. (severity: major, 2026-04-06)
- [ ] [rendering] Remove radius dithering for circles (severity: minor, 2026-04-06)
- [ ] [rendering] Soft falloff for all SDF-based alpha — pixel-sized smoothstep, but falloff width configurable. Smoke: 20% falloff (full opacity at 80% diameter, 0% at 100% spawn diameter) (severity: major, 2026-04-06)
- [ ] [rendering] MSAA is useless for SDF-based effects — document this, remove MSAA from affected passes, plan pass-aware AA strategy (severity: minor, 2026-04-06)
- [ ] [rendering] Add SSAA modes: 4×, 9×, 16× — for evaluating AA quality and scale-independence (severity: minor, 2026-04-06)
- [ ] [rendering] MSDF text rendering — replace polygon-based glyph system with MSDF for better quality and scalability (severity: minor, 2026-04-03)

### Brightness & Color Correctness

- [ ] [rendering] Mathematically exact brightness conservation for SDF capsules — brightness×area must remain constant as circle becomes streak. Focus circle→capsule first; soft attenuation deferred (severity: major, 2026-04-06)
- [ ] [rendering] Global color effects (screen flash, damage tint, slow-mo color shift) must be post-process. Per-object color (asteroid tint, bullet color) stays in forward/instance pass — do not move these to post-process (severity: major, 2026-04-06)
- [ ] [rendering] Use widest available color gamut — ship should render at maximum displayable red (severity: minor, 2026-04-06)
- [ ] [rendering/hdr] HUD brightness: reach target nits, then skip tonemapping if below max_brightness, otherwise apply same tonemap as scene. Current behavior is dull/washed-out (severity: major, 2026-04-06)
- [ ] [rendering/hdr] Gamut mapping when display gamut info is available (sRGB display gets gamut-mapped P3 colors) (severity: minor, 2026-04-04)

---

## Camera

- [ ] [camera] Dezoom to keep ship on screen — begin unzooming when ship approaches 25% from safe zone border, accelerate closer to edge. Physical inertia behavior (severity: major, 2026-04-06)
  > Note: was planned for Phase 3 but requested immediately — pull forward.

---

## Particles & Visual Effects

- [ ] [particles] Smoke inherits explosion speed; ship fire smoke inherits ship velocity (severity: minor, 2026-04-06)
- [ ] [particles] Ship fire must inherit ship velocity + added speed opposite to acceleration direction. Fixes engine fire not visible at high speeds. (severity: minor, 2026-03-28)

---

## Physics & Architecture

- [ ] [architecture] Distortion field system — analytical field evaluation replacing one-shot shockwave, expanding ring shockwaves, gravity wells, time dilation via proper_time. Spec: `docs/superpowers/specs/2026-04-01-distortion-field-design.md` (severity: major, 2026-04-03)
  > Prerequisite for gravity wells and time dilation game mechanics.

---

## i18n & Glyph System

- [ ] [i18n] Internationalization system — locale files (RON), system locale detection, string extraction from HUD/pause menu. Spec: `docs/superpowers/specs/2026-04-03-i18n-locale-glyph-design.md` (severity: minor, 2026-04-03)
- [ ] [rendering] Extended glyph system — 26 lowercase glyphs, 5 accent marks, ~15 composed French chars, three-tier lookup with override support. Spec: same as above (severity: minor, 2026-04-03)
- [ ] [i18n] Western European chars (scope B) — German ß/ä/ö/ü, Spanish ¿/¡, Scandinavian å/æ/ø, Portuguese ã/õ (~30 glyphs) (severity: minor, 2026-04-03)
- [ ] [i18n] Full Latin Extended chars (scope C) — Polish, Czech, Hungarian, 50+ glyphs (severity: minor, 2026-04-03)
  > Scopes B and C blocked on scope A (extended glyph system) and MSDF decision.

---

## Haptics

- [ ] [haptics] Gamepad vibration system — configurable, any game event as vibration source (severity: minor, 2026-04-06)
- [ ] [haptics] Advanced haptics research & plan — DualSense adaptive triggers/HD haptics, Switch HD Rumble, Xbox impulse triggers; document what's available per platform (severity: minor, 2026-04-06)
  > Plan haptics API surface before audio so both can share the "game event" abstraction.

---

## Audio

- [ ] [audio] Synthesized audio engine — everything computed/synthesized in real-time; sample-based sounds as fallback. Engine should flexibly mix synthesis methods: additive, FM, wavetable, physical modeling — not locked to a single approach (severity: major, 2026-04-06)
- [ ] [audio] Explosion and game event sounds — full sound design plan, map events to synthesis parameters (severity: major, 2026-04-06)
- [ ] [audio] MIDI as data format — primarily for defining game music and sound triggers in composition files. Live MIDI input as a stretch goal (severity: minor, 2026-04-06)
- [ ] [audio] Custom track/music format — composition format for tracks and music (severity: minor, 2026-04-06)
- [ ] [audio] Dynamic audio effects — global, per-track/instrument, per-note: flanger, reverb, Paul stretch (real-time live effect, not offline asset processing), etc. (severity: minor, 2026-04-06)
- [ ] [audio] Slow-mo affects sound — pitch drop, granular time stretch tied to simulation time scale (severity: minor, 2026-04-06)
  > Audio slow-mo requires the fixed-dt/time-scale infrastructure from Phase 0 — already in place.

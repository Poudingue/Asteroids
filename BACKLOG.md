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

- [ ] [rendering] Merge "exposure" and "game exposure" into single "exposure" slider � game exposure is internal state only (affected by events), not user-adjustable. Remove separate game_exposure slider from pause menu (severity: major, 2026-04-07)
- [ ] [rendering/hdr] Clarify max_brightness role: it IS the tonemap threshold � for pseudo-Reinhard it's the target at +infinity input. Ensure all tonemap variants use it correctly as their threshold parameter (severity: major, 2026-04-07)

- [x] [rendering] Bind group layout duplication in Renderer2D::resize() — discovered during code review, removed redundant layouts (severity: minor, 2026-04-08)
- [ ] [rendering] Document that MSAA only affects polygon geometry (ship/asteroid edges), not SDF circles/capsules. MSAA stays for polygon AA (severity: minor, 2026-04-06)
- [ ] [rendering] SMAA post-process AA — optional pass between scene render and final output. Complements MSAA (geometry) and SSAA (supersample). All three independently toggleable and stackable. Pipeline order: MSAA scene -> SMAA post-process -> SSAA downsample (severity: major, 2026-04-07)
- [ ] [rendering] SSAA modes (4x, 9x, 16x) — render offscreen at higher resolution, downsample in postprocess. Purpose: generate pixel-perfect reference renders for visual comparison via screenshot tools. Optional, default Off (severity: major, 2026-04-06)
- [ ] [rendering] MSDF text rendering — replace polygon-based glyph system with MSDF for better quality and scalability (severity: minor, 2026-04-03)
- [ ] [rendering] Velocity buffer (motion vectors) — per-pixel screen-space velocity render target (Rg16Float), written in world+SDF passes. Enables TAA/TSSAA temporal reprojection, post-process motion blur, wind visual effects, GPU particle trails. Prerequisite for temporal AA and advanced post-process effects (severity: major, 2026-04-07)

### Brightness & Color Correctness

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

## Controls & Gameplay

- [ ] [controls] Gamepad: orienting ship should trigger continuous shooting by default (severity: major, 2026-04-07)
- [ ] [controls] Gamepad: left button triggers teleport+explosion (was: separate action) (severity: major, 2026-04-07)
- [ ] [controls] KB+Mouse: right-click triggers teleport+explosion (severity: major, 2026-04-07)
- [ ] [gameplay] Teleport invulnerability: ship should not be damaged by the asteroid it teleports into (or its fragments) until they move far enough to become "normal" objects � similar to explosion fragment grace period (severity: major, 2026-04-07)

---

## Haptics

- [ ] [ui] Pause menu should be navigable via gamepad (severity: minor, 2026-04-07)

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

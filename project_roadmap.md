# Asteroids V2 — Project Roadmap

## Status Legend
- ✅ Complete
- 🔄 In progress / partially done
- ⬜ Planned

---

## Phase 0 — Foundation ✅
Entity model, ECS-lite architecture, RON scenario files, fixed-dt deterministic loop, gamepad input, world-space controls.

## Phase 1 — Rendering Foundation ✅
wgpu pipeline, SDF shapes, HDR surface (sRGB ↔ Rgba16Float), tonemap variants (Passthrough, Pseudo-Reinhard, Hard/Soft Redirect), pause menu redesign, MSAA toggle, HUD brightness uniforms.

> Visual testing of Phases 0–1 still outstanding — see BACKLOG.md.

---

## Phase 2 — Rendering Correctness ⬜
Fix the rendering pipeline before adding more features on top of a shaky foundation.

**Goals:**
- Layer order: background → stars → smoke → chunks → sparkles (collision light-trails, intensity ∝ collision force) → bullets → asteroids → explosions → ship
- Mutualize stars/bullets/sparkles (configurable capsule, shutter speed, falloff). Shutter speed: 0%=circle, 100%=physical trail, >100%=exaggerated
- Mathematically exact brightness conservation: brightness×area constant through circle→capsule morph
- Soft SDF alpha falloff — pixel-sized smoothstep, configurable width; smoke-specific 20% falloff
- Global color effects (screen flash, damage tint, slow-mo shift) post-process; per-object color (asteroid tint, bullet color) stays in forward/instance pass
- Widest gamut colors; ship = max displayable red
- HUD brightness fix: target nits → skip tonemap if below max, else apply scene tonemap
- Smoke velocity inheritance (explosion speed, ship velocity for engine fire)
- Remove radius dithering for circles
- Remove MSAA from SDF passes; add SSAA modes (4×, 9×, 16×) for evaluation
- Engine fire ejection fix at high speeds
- **Testing & capture tooling**: screenshot tool (A/B comparison across pre-recorded scenarios), video capture (rendering regression), scenario recording UI ("Record Scenario" checkbox → fixed-dt + input+state recording → deterministic replay file). Builds on Phase 0 fixed-dt and RON/zstd+bincode recording infrastructure.

**Dependencies:** Phase 1 complete ✅

---

## Phase 3 — Camera & Zoom ⬜
Dynamic dezoom to keep ship on screen.

**Goals:**
- Begin unzooming when ship is within 25% of safe zone border
- Accelerate unzooming as ship approaches edge
- Physical inertia — camera zoom behaves like a damped spring

**Note:** Originally planned later but pulled forward — blocking good gameplay feel. Can overlap with Phase 2 (independent systems).

**Dependencies:** Phase 0 ✅

---

## Phase 4 — Physics ⬜
Replace circle collisions with proper polygon geometry.

**Goals:**
- parry2d integration for polygon collisions
- Fix mass gain on fragmentation, conserve momentum/energy
- Distortion field system — analytical fields, expanding ring shockwaves, gravity wells, time dilation via proper_time (spec: `docs/superpowers/specs/2026-04-01-distortion-field-design.md`)

**Dependencies:** Phase 0 ✅, Phase 2 recommended (rendering must be stable before physics overhaul)

---

## Phase 5 — GPU Particles ⬜
Move particle systems to compute shaders.

**Goals:**
- Compute shader particle simulation
- High-count sparkles, smoke, debris without CPU bottleneck

**Dependencies:** Phase 2 (rendering correctness), Phase 4 (physics for particle spawn events)

---

## Phase 6 — Weapons ⬜
Combat depth.

**Goals:**
- 3 weapon types
- Scroll wheel weapon select
- HUD weapon indicator

**Dependencies:** Phase 4 (physics for hit detection)

---

## Phase 7 — i18n & Glyph System ⬜
Localization infrastructure and extended character support.

**Goals:**
- RON locale files, system locale detection, string extraction
- Extended glyph system: lowercase, accents, French chars (scope A)
- Western European scope B (~30 glyphs)
- Full Latin Extended scope C (50+ glyphs)
- Evaluate MSDF text rendering as long-term replacement for polygon glyphs

**Dependencies:** Phase 2 (rendering pipeline stable), no gameplay dependency

**Note:** Can be worked in parallel with Phases 4–6 if bandwidth allows.

---

## Phase 8 — Haptics ⬜
Gamepad feedback.

**Goals:**
- Configurable vibration system — any game event as vibration source
- Advanced haptics research: DualSense adaptive triggers/HD haptics, Switch HD Rumble, Xbox impulse triggers
- Document platform-specific APIs and capability matrix

**Note:** Design the "game event" abstraction here — Audio (Phase 9) will share it.

**Dependencies:** Phase 0 ✅ (gamepad input)

---

## Phase 9 — Audio ⬜
Full synthesized audio engine.

**Goals:**
- Real-time synthesis engine, mixing methods freely (additive, FM, wavetable, physical modeling); sample-based fallback
- Explosion and game event sounds — full sound design mapped to synthesis params
- Slow-mo sound: pitch drop + granular time stretch tied to simulation time scale
- Dynamic effects: flanger, reverb, Paul stretch (real-time live effect) — global, per-track, per-note
- MIDI primarily as data format for defining music/sound triggers; live MIDI input as stretch goal
- Custom track/music composition format

**Note:** Slow-mo audio depends on fixed-dt time-scale infrastructure — already in place from Phase 0. ✅
Game event abstraction shared with Haptics (Phase 8) — design together or sequence Phase 8 first.

**Dependencies:** Phase 0 ✅, Phase 8 (shared event abstraction)

---

## HDR Output — Ongoing 🔄
Not a separate phase — HDR work is threaded through Phases 1–2.

- Phase 1: Surface switching, tonemap variants, uniforms ✅
- Phase 2: HUD brightness fix, gamut correctness, SSAA
- Future: Gamut mapping when display gamut metadata available

---

## Rough Sequence

```
Phase 0 ✅ → Phase 1 ✅ → Phase 2 → Phase 3 (parallel ok)
                                  ↓
                              Phase 4 → Phase 5 → Phase 6
                                  ↓
                    Phase 7 (parallel with 4–6)
                    Phase 8 → Phase 9
```

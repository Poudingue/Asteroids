# Internationalization & Extended Glyph System — Design Spec

**Date:** 2026-04-03
**Scope:** Locale file system for multi-language support, extended glyph library with lowercase and accented characters, system locale auto-detection.

---

## 1. Locale System

### File Format

RON files, one per language: `locales/en.ron`, `locales/fr.ron`, etc.

```ron
Locale(
    name: "Français",
    code: "fr",
    strings: {
        "score_label": "SCORE",
        "stage_label": "NIVEAU",
        "pause_title": "ASTEROIDS",
        "btn_quit": "quitter",
        "btn_quit_tip": "Quitter le jeu et aller dehors",
        "btn_resume": "reprendre",
        "btn_resume_tip": "Reprendre la partie en cours",
        "btn_new_game": "Nouvelle partie",
        "btn_new_game_tip": "Commencer une nouvelle partie",
        "btn_advanced_hitbox": "Hitbox avancée",
        "btn_advanced_hitbox_tip": "Une hitbox plus précise.",
        "btn_smoke": "particules de fumée",
        "btn_smoke_tip": "Active la fumée. Désactiver pour de meilleures performances.",
        "btn_screenshake": "tremblement",
        "btn_screenshake_tip": "Ressentir les impacts et explosions.",
        "btn_flashes": "Flashs lumineux",
        "btn_flashes_tip": "Active les flashs lumineux pour les événements",
        "btn_chunks": "particules de débris",
        "btn_chunks_tip": "Active les débris. Désactiver pour de meilleures performances.",
        "btn_color_effects": "Effets de couleur",
        "btn_color_effects_tip": "Changements et correction de couleur",
        "debug_fps": "FPS",
        "debug_peak_fps": "Peak FPS",
        "debug_objects": "Objets",
        "debug_toosmall": "TooSmall",
        "debug_frags": "Frags",
        "debug_projectiles": "Projectiles",
        "debug_explosions": "Explosions",
        "debug_smoke": "Smoke",
        "debug_chunks": "Chunks",
        "debug_chunks_explo": "ChunksExplo",
        "teleport_ready": "F",
    },
)
```

### Locale Selection Priority

1. `--lang fr` CLI flag (highest priority)
2. System locale auto-detection via `sys-locale` crate (returns e.g. `"fr-FR"`, take language prefix)
3. English fallback (always loaded)

### Module: `src/locale.rs`

```rust
pub struct Locale {
    pub name: String,
    pub code: String,
    pub strings: HashMap<String, String>,
}
```

- `Locale::load(path) -> Result<Locale, String>` — load from RON file
- `Locale::get(key) -> &str` — lookup with English fallback
- `detect_system_locale() -> Option<String>` — returns language code from OS
- English locale is always loaded as fallback. Active locale overlays it.

### New Dependency

```toml
sys-locale = "0.3"
```

### Future Extensions (backlog, not implemented now)

- Dark theme detection via OS settings (Windows registry / `dark-light` crate)
- Additional locale files for Western European languages (scope B)
- Full Latin Extended support (scope C)

---

## 2. Extended Glyph System

### Current State

- `shape_char(c: char) -> Vec<(f64, f64)>` in `src/rendering/hud.rs`
- 31 characters: A-Z, 0-9, `: - . !`, space
- Single closed polygon per char in [0,1]² unit space
- `render_string` coerces all input to uppercase via `to_ascii_uppercase()`
- Even-odd scanline fill

### New Characters (Scope A: French + English)

**26 lowercase glyphs** (hand-designed, proper typographic proportions):
- Standard x-height letters: a, c, e, m, n, o, r, s, u, v, w, x, z
- Ascenders (extend above x-height): b, d, f, h, k, l, t
- Descenders (extend below baseline): g, j, p, q, y
- Special: i (dot above)

**5 accent mark shapes** (reusable, positioned above or below):
- Acute: ´ (rising stroke, positioned above)
- Grave: ` (falling stroke, positioned above)
- Circumflex: ^ (inverted V, positioned above)
- Diaeresis: ¨ (two dots, positioned above)
- Cedilla: ¸ (hook, positioned below baseline)

**~15 composed accented characters** (French set, both cases):
- É/é, È/è, Ê/ê, Ë/ë
- À/à, Â/â
- Ù/ù, Û/û, Ü/ü
- Ô/ô, Ö/ö (Ö for borrowed words)
- Î/î, Ï/ï
- Ç/ç
- Ñ/ñ (for borrowed words)

**Additional punctuation** (needed for French):
- « » (guillemets — French quotation marks)
- ? (question mark — not currently in charset)
- ' (apostrophe)

**Total: ~50 new glyph definitions.**

### Glyph Lookup: Three-Tier System

```
1. Override table  →  custom standalone glyph for this exact char
2. Composition     →  base letter + accent mark, paths concatenated
3. shape_char      →  existing match arm (uppercase, digits, punctuation)
4. Fallback        →  filled square
```

**Override table**: `HashMap<char, Vec<(f64, f64)>>`, populated at init. Allows hand-crafting any composed char that doesn't look right with automatic composition. Example: if composed `é` looks wrong, add a custom override — no code change needed.

**Composition**: Define accent marks with a position parameter:

```rust
pub struct AccentDef {
    pub shape: Vec<(f64, f64)>,  // polygon in [0,1]² accent space
    pub placement: AccentPlacement,
}

pub enum AccentPlacement {
    Above,  // acute, grave, circumflex, diaeresis
    Below,  // cedilla
}
```

At init, for each accented char:
1. Look up base letter polygon (e.g., `'e'` for `'é'`)
2. Look up accent shape (e.g., acute)
3. Scale and position accent relative to base letter (above x-height or below baseline)
4. Concatenate both polygon paths into a single polygon
5. Store in composed glyph cache

**Lowercase coordinate conventions**:
- x-height: y ∈ [0.0, 0.65] (65% of cell height)
- Ascender height: y ∈ [0.0, 1.0] (full cell)
- Descender depth: y ∈ [-0.2, 0.65] (extends below baseline)
- Accent zone (above): y ∈ [0.7, 1.0]
- Cedilla zone (below): y ∈ [-0.15, 0.0]
- Uppercase letters remain at y ∈ [0.0, 1.0] as currently

### New Module: `src/glyphs.rs`

Extracted from `hud.rs` to keep rendering focused:

- `shape_char(c: char) -> Vec<(f64, f64)>` — moved here from hud.rs
- Lowercase glyph definitions (a-z)
- Accent mark definitions
- Composition logic (base + accent → combined polygon)
- Override table (`HashMap<char, Vec<(f64, f64)>>`)
- `pub fn glyph(c: char) -> Vec<(f64, f64)>` — top-level lookup function implementing the three-tier system

### render_string Change

- Remove `to_ascii_uppercase()` coercion
- Replace `shape_char(c)` call with `glyphs::glyph(c)` call
- No other rendering changes needed — the scanline fill, shake, bilinear interpolation all work unchanged

---

## 3. Integration with Existing Code

### String Key Usage

All current inline string literals in `hud.rs` and `pause_menu.rs` get replaced with locale lookups:

```rust
// Before:
render_string(&format!("SCORE {}", state.score), ...);

// After:
render_string(&format!("{} {}", locale.get("score_label"), state.score), ...);
```

The `Locale` reference is passed through `Globals` or as a separate parameter to render functions.

### Pause Menu Buttons

`ButtonBoolean` currently stores `text: &'static str` and `text_over: &'static str`. These become locale keys resolved at render time (not at button creation), so language can be switched without recreating buttons.

### CLI Integration

Add to existing `Cli` struct in `main.rs`:

```rust
/// Language code (e.g., "fr", "en"). Auto-detected if not specified.
#[arg(long)]
lang: Option<String>,
```

---

## 4. File Structure

| File | Role |
|------|------|
| `src/locale.rs` | **New** — Locale struct, RON loading, key lookup, system detection |
| `src/glyphs.rs` | **New** — Glyph definitions (lowercase, accents), composition, override table, lookup |
| `locales/en.ron` | **New** — English string table (canonical keys) |
| `locales/fr.ron` | **New** — French string table |
| `src/rendering/hud.rs` | **Modify** — Replace inline strings with locale lookups, use `glyphs::glyph()` |
| `src/pause_menu.rs` | **Modify** — Replace inline strings with locale keys |
| `src/main.rs` | **Modify** — Add `--lang` CLI arg, load locale |
| `src/lib.rs` | **Modify** — Export new modules |
| `Cargo.toml` | **Modify** — Add `sys-locale` dependency |

---

## 5. Implementation Order

1. **Locale system** — `locale.rs`, RON files, CLI flag, system detection, English fallback
2. **String extraction** — Replace all inline strings in `hud.rs` and `pause_menu.rs` with locale key lookups
3. **Glyph infrastructure** — `glyphs.rs` with three-tier lookup, accent definitions, composition logic
4. **Lowercase glyphs** — Hand-design 26 lowercase polygon glyphs (a-z)
5. **Accented characters** — Define 5 accent marks, compose ~15 French accented chars (both cases)
6. **French locale** — Write `locales/fr.ron` with all translated strings
7. **Additional punctuation** — «, », ?, '

---

## 6. Testing

- Unit test: locale loading, key lookup, fallback behavior
- Unit test: glyph composition (base + accent produces valid polygon)
- Unit test: three-tier lookup order (override > compose > shape_char > fallback)
- Scenario test: determinism not affected (locale is render-only, no physics impact)
- Visual testing: each new glyph renders correctly (requires screen — backlog item)

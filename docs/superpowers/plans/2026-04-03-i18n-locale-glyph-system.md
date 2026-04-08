# i18n Locale & Extended Glyph System — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add multi-language locale system (RON files, CLI `--lang`, OS auto-detection, English fallback) and extended glyph library (26 lowercase, 5 accents, ~15 composed French chars, extra punctuation) to support French UI. Scope A only (French + English).

**Architecture:** Two new modules: `src/locale.rs` (locale loading, key lookup, system detection) and `src/glyphs.rs` (glyph definitions, three-tier lookup, accent composition). Locale strings replace all hardcoded text in HUD and pause menu. The `render_string` function delegates to `glyphs::glyph()` instead of the local `shape_char`, removing the `to_ascii_uppercase()` coercion.

**Tech Stack:** Rust, RON (already a dependency), sys-locale crate (new), existing polygon rendering

**Design spec:** `docs/superpowers/specs/2026-04-03-i18n-locale-glyph-design.md`

---

## File Structure

| File | Role |
|------|------|
| `src/locale.rs` | **New** — `Locale` struct, RON loading, key lookup with fallback, `detect_system_locale()` |
| `src/glyphs.rs` | **New** — Glyph definitions (lowercase a-z), accent marks, composition, override table, `glyph()` top-level lookup |
| `locales/en.ron` | **New** — English string table (canonical keys, all values) |
| `locales/fr.ron` | **New** — French string table (translated values) |
| `src/lib.rs` | **Modify** — Export `locale` and `glyphs` modules |
| `src/main.rs` | **Modify** — Add `--lang` CLI arg, load locale, pass to `Globals` |
| `src/parameters.rs` | **Modify** — Add `locale: Locale` field to `Globals` |
| `src/rendering/hud.rs` | **Modify** — Move `shape_char` to `glyphs.rs`, use `glyphs::glyph()`, replace hardcoded strings with locale lookups |
| `src/pause_menu.rs` | **Modify** — Replace hardcoded button text/tooltips with locale key resolution at render time |
| `Cargo.toml` | **Modify** — Add `sys-locale = "0.3"` dependency |
| `tests/locale_tests.rs` | **New** — Unit tests for locale loading, fallback, glyph lookup |

---

## Task 1: Add `sys-locale` dependency and create `locale.rs` skeleton

**Goal:** Create the Locale struct with RON loading and key lookup. English fallback built in.

### Step 1.1: Add dependency

- [ ] In `Cargo.toml`, add `sys-locale`:

```toml
sys-locale = "0.3"
```

- [ ] Run:

```bash
rtk cargo check
```

### Step 1.2: Create `src/locale.rs` with Locale struct and loading

- [ ] Create `src/locale.rs`:

```rust
//! Internationalization locale system.
//!
//! RON-based string tables with OS locale auto-detection and English fallback.

use std::collections::HashMap;

/// A loaded locale: name, language code, and string table.
#[derive(Debug, Clone)]
pub struct Locale {
    pub name: String,
    pub code: String,
    pub strings: HashMap<String, String>,
}

/// Raw RON structure for deserialization.
#[derive(Debug, serde::Deserialize)]
struct LocaleFile {
    name: String,
    code: String,
    strings: HashMap<String, String>,
}

impl Locale {
    /// Load a locale from a RON file at the given path.
    pub fn load(path: &str) -> Result<Locale, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
        let file: LocaleFile =
            ron::from_str(&content).map_err(|e| format!("Failed to parse {}: {}", path, e))?;
        Ok(Locale {
            name: file.name,
            code: file.code,
            strings: file.strings,
        })
    }

    /// Look up a string key. Returns the value if present, or the key itself as fallback.
    pub fn get(&self, key: &str) -> &str {
        self.strings
            .get(key)
            .map(|s| s.as_str())
            .unwrap_or(key)
    }

    /// Create a combined locale: `active` overlays `fallback`.
    /// Keys present in `active` take priority; missing keys fall through to `fallback`.
    pub fn with_fallback(active: Locale, fallback: &Locale) -> Locale {
        let mut strings = fallback.strings.clone();
        strings.extend(active.strings);
        Locale {
            name: active.name,
            code: active.code,
            strings,
        }
    }
}

/// Detect the system locale and return the language code (e.g. "fr", "en").
/// Returns `None` if detection fails.
pub fn detect_system_locale() -> Option<String> {
    let raw = sys_locale::get_locale()?;
    // sys-locale returns e.g. "fr-FR", "en-US", "fr_FR". Take the prefix before '-' or '_'.
    let code = raw
        .split(|c| c == '-' || c == '_')
        .next()?
        .to_lowercase();
    if code.is_empty() {
        None
    } else {
        Some(code)
    }
}

/// Resolve the active locale given CLI override and system detection.
/// Priority: cli_lang > system locale > "en".
/// Loads from `locales/{code}.ron`, falling back to English.
pub fn resolve_locale(cli_lang: Option<&str>) -> Locale {
    let en_path = "locales/en.ron";
    let fallback = Locale::load(en_path).unwrap_or_else(|e| {
        eprintln!("Warning: failed to load English locale: {}. Using empty fallback.", e);
        Locale {
            name: "English".to_string(),
            code: "en".to_string(),
            strings: HashMap::new(),
        }
    });

    let lang_code = cli_lang
        .map(|s| s.to_string())
        .or_else(detect_system_locale)
        .unwrap_or_else(|| "en".to_string());

    if lang_code == "en" {
        return fallback;
    }

    let path = format!("locales/{}.ron", lang_code);
    match Locale::load(&path) {
        Ok(active) => Locale::with_fallback(active, &fallback),
        Err(e) => {
            eprintln!("Warning: failed to load locale '{}': {}. Using English.", lang_code, e);
            fallback
        }
    }
}
```

### Step 1.3: Export module in `lib.rs`

- [ ] In `src/lib.rs`, add `pub mod locale;` after the existing module declarations.

### Step 1.4: Verify compilation

- [ ] Run:

```bash
rtk cargo check
```

**Expected:** Clean compilation, no errors.

### Step 1.5: Commit

```bash
rtk git add src/locale.rs src/lib.rs Cargo.toml && rtk git commit -m "feat: locale module with RON loading, fallback, and system detection"
```

---

## Task 2: Create English and French locale RON files

**Goal:** Create the two RON string tables with all keys used in HUD and pause menu.

### Step 2.1: Create `locales/` directory and `locales/en.ron`

- [ ] Create `locales/en.ron`:

```ron
Locale(
    name: "English",
    code: "en",
    strings: {
        "score_label": "SCORE",
        "stage_label": "STAGE",
        "pause_title": "ASTEROIDS",
        "btn_quit": "quit",
        "btn_quit_tip": "Quit the game and go outside",
        "btn_resume": "resume",
        "btn_resume_tip": "Resume current game",
        "btn_new_game": "New Game",
        "btn_new_game_tip": "Start a new game with current parameters",
        "btn_advanced_hitbox": "Advanced hitbox",
        "btn_advanced_hitbox_tip": "A more precise hitbox.",
        "btn_smoke": "smoke particles",
        "btn_smoke_tip": "Allows smoke. Disable for better performance.",
        "btn_screenshake": "screenshake",
        "btn_screenshake_tip": "Feel the impacts and explosions.",
        "btn_flashes": "Light Flashes",
        "btn_flashes_tip": "Activates light flashes for events",
        "btn_chunks": "chunk particles",
        "btn_chunks_tip": "Allows chunks. Disable for better performance.",
        "btn_color_effects": "Color Effects",
        "btn_color_effects_tip": "Color changes and correction",
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

### Step 2.2: Create `locales/fr.ron`

- [ ] Create `locales/fr.ron` (ASCII-only for now; accented chars added in Task 8 once glyphs support them):

```ron
Locale(
    name: "Francais",
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
        "btn_advanced_hitbox": "Hitbox avancee",
        "btn_advanced_hitbox_tip": "Une hitbox plus precise.",
        "btn_smoke": "particules de fumee",
        "btn_smoke_tip": "Active la fumee. Desactiver pour de meilleures performances.",
        "btn_screenshake": "tremblement",
        "btn_screenshake_tip": "Ressentir les impacts et explosions.",
        "btn_flashes": "Flashs lumineux",
        "btn_flashes_tip": "Active les flashs lumineux pour les evenements",
        "btn_chunks": "particules de debris",
        "btn_chunks_tip": "Active les debris. Desactiver pour de meilleures performances.",
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

### Step 2.3: Verify locale loading compiles

- [ ] Create `tests/locale_tests.rs`:

```rust
use asteroids::locale::Locale;

#[test]
fn test_load_english_locale() {
    let locale = Locale::load("locales/en.ron").expect("Failed to load English locale");
    assert_eq!(locale.code, "en");
    assert_eq!(locale.get("score_label"), "SCORE");
    assert_eq!(locale.get("stage_label"), "STAGE");
}

#[test]
fn test_load_french_locale() {
    let locale = Locale::load("locales/fr.ron").expect("Failed to load French locale");
    assert_eq!(locale.code, "fr");
    assert_eq!(locale.get("stage_label"), "NIVEAU");
}

#[test]
fn test_fallback_returns_key_for_missing() {
    let locale = Locale::load("locales/en.ron").expect("Failed to load English locale");
    // Non-existent key returns the key itself
    assert_eq!(locale.get("nonexistent_key"), "nonexistent_key");
}

#[test]
fn test_with_fallback_overlay() {
    let en = Locale::load("locales/en.ron").expect("Failed to load English locale");
    let fr = Locale::load("locales/fr.ron").expect("Failed to load French locale");
    let combined = Locale::with_fallback(fr, &en);
    // French override
    assert_eq!(combined.get("stage_label"), "NIVEAU");
    // Shared key present in both
    assert_eq!(combined.get("score_label"), "SCORE");
    assert_eq!(combined.code, "fr");
}
```

- [ ] Run:

```bash
rtk cargo test --test locale_tests
```

**Expected:** All 4 tests pass.

### Step 2.4: Commit

```bash
rtk git add locales/ tests/locale_tests.rs && rtk git commit -m "feat: English and French locale RON files with unit tests"
```

---

## Task 3: Integrate locale into Globals and CLI

**Goal:** Add `--lang` CLI flag, load locale at startup, store in `Globals` for use by HUD and pause menu.

### Step 3.1: Add locale to `Globals`

- [ ] In `src/parameters.rs`, add import at the top:

```rust
use crate::locale::Locale;
```

- [ ] Add field to `Globals` struct (after `observer_proper_time`):

```rust
pub locale: Locale,
```

- [ ] Update `Globals::new()` to initialize with a default empty English locale:

```rust
locale: Locale {
    name: "English".to_string(),
    code: "en".to_string(),
    strings: std::collections::HashMap::new(),
},
```

(This will be overwritten by `resolve_locale` in `main.rs` before the game loop starts.)

### Step 3.2: Add `--lang` CLI flag in `main.rs`

- [ ] In `src/main.rs`, add to the `Cli` struct:

```rust
/// Language code (e.g., "fr", "en"). Auto-detected if not specified.
#[arg(long)]
lang: Option<String>,
```

- [ ] After `globals.recompute_for_resolution(width, height);` (around line 75), add locale loading:

```rust
globals.locale = asteroids::locale::resolve_locale(cli.lang.as_deref());
println!("Locale: {} ({})", globals.locale.name, globals.locale.code);
```

### Step 3.3: Verify compilation

- [ ] Run:

```bash
rtk cargo check
```

**Expected:** Clean compilation.

### Step 3.4: Run all tests

- [ ] Run:

```bash
rtk cargo test
```

**Expected:** All existing tests + locale tests pass.

### Step 3.5: Commit

```bash
rtk git add src/parameters.rs src/main.rs && rtk git commit -m "feat: --lang CLI flag, locale loaded into Globals at startup"
```

---

## Task 4: Create `glyphs.rs` with three-tier lookup infrastructure

**Goal:** Extract `shape_char` from `hud.rs` into `glyphs.rs`, add the three-tier lookup system (override -> compose -> shape_char -> fallback), and wire it up. Includes all new glyph shapes.

### Step 4.1: Create `src/glyphs.rs` with extracted `shape_char` and lookup skeleton

- [ ] Create `src/glyphs.rs` with the following content:

```rust
//! Glyph definitions and three-tier lookup system.
//!
//! Tier 1: Override table -- custom standalone glyph for exact char
//! Tier 2: Composition -- base letter + accent mark, paths concatenated
//! Tier 3: shape_char -- existing match arms (uppercase, digits, punctuation)
//! Tier 4: Fallback -- filled square

use std::collections::HashMap;

/// Accent mark placement relative to the base glyph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccentPlacement {
    /// Positioned above the letter (acute, grave, circumflex, diaeresis)
    Above,
    /// Positioned below the baseline (cedilla)
    Below,
}

/// An accent mark definition: shape polygon and placement.
#[derive(Debug, Clone)]
pub struct AccentDef {
    pub shape: Vec<(f64, f64)>,
    pub placement: AccentPlacement,
}

/// Decomposition of an accented character into base + accent.
#[derive(Debug, Clone)]
struct Decomposition {
    base: char,
    accent_name: &'static str,
}

/// The glyph cache, built once at startup.
pub struct GlyphCache {
    /// Tier 1: Hand-crafted overrides for specific characters.
    overrides: HashMap<char, Vec<(f64, f64)>>,
    /// Tier 2: Composed glyphs (base + accent), cached after first build.
    composed: HashMap<char, Vec<(f64, f64)>>,
    /// Accent mark definitions by name.
    accents: HashMap<&'static str, AccentDef>,
    /// Decomposition table: accented char -> (base, accent_name).
    decompositions: HashMap<char, Decomposition>,
}

impl GlyphCache {
    /// Build the glyph cache with all accent definitions and decompositions.
    pub fn new() -> Self {
        let accents = Self::build_accents();
        let decompositions = Self::build_decompositions();
        let mut cache = GlyphCache {
            overrides: HashMap::new(),
            composed: HashMap::new(),
            accents,
            decompositions,
        };
        cache.precompose_all();
        cache
    }

    /// Top-level glyph lookup implementing the three-tier system.
    /// Returns a polygon in relative coordinates.
    pub fn glyph(&self, c: char) -> Vec<(f64, f64)> {
        // Tier 1: Override
        if let Some(shape) = self.overrides.get(&c) {
            return shape.clone();
        }
        // Tier 2: Composed (precomputed)
        if let Some(shape) = self.composed.get(&c) {
            return shape.clone();
        }
        // Tier 3: shape_char (uppercase, digits, punctuation, lowercase)
        let shape = shape_char(c);
        if !shape.is_empty() {
            return shape;
        }
        // Tier 4: Fallback -- filled square
        vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
    }

    fn build_accents() -> HashMap<&'static str, AccentDef> {
        let mut m = HashMap::new();
        // Acute: rising stroke, positioned in accent zone y=[0.70, 0.95]
        m.insert("acute", AccentDef {
            shape: vec![(0.35, 0.70), (0.65, 0.95), (0.55, 0.95), (0.25, 0.70)],
            placement: AccentPlacement::Above,
        });
        // Grave: falling stroke
        m.insert("grave", AccentDef {
            shape: vec![(0.35, 0.95), (0.65, 0.70), (0.75, 0.70), (0.45, 0.95)],
            placement: AccentPlacement::Above,
        });
        // Circumflex: inverted V
        m.insert("circumflex", AccentDef {
            shape: vec![
                (0.25, 0.75), (0.5, 0.95), (0.75, 0.75),
                (0.65, 0.75), (0.5, 0.88), (0.35, 0.75),
            ],
            placement: AccentPlacement::Above,
        });
        // Diaeresis: two dots
        m.insert("diaeresis", AccentDef {
            shape: vec![
                // Left dot
                (0.25, 0.80), (0.40, 0.80), (0.40, 0.92), (0.25, 0.92),
                // Right dot (separate closed region -- even-odd fill renders both)
                (0.60, 0.80), (0.75, 0.80), (0.75, 0.92), (0.60, 0.92),
                // Bridge back to left dot to close the path
                (0.60, 0.80), (0.40, 0.80),
            ],
            placement: AccentPlacement::Above,
        });
        // Cedilla: hook below baseline
        m.insert("cedilla", AccentDef {
            shape: vec![
                (0.40, 0.0), (0.60, 0.0),
                (0.55, -0.05), (0.60, -0.12),
                (0.50, -0.18), (0.35, -0.12),
                (0.45, -0.08), (0.45, -0.02),
            ],
            placement: AccentPlacement::Below,
        });
        // Tilde: for n-tilde
        m.insert("tilde", AccentDef {
            shape: vec![
                (0.20, 0.80), (0.35, 0.92), (0.50, 0.85), (0.65, 0.92), (0.80, 0.80),
                (0.65, 0.85), (0.50, 0.78), (0.35, 0.85),
            ],
            placement: AccentPlacement::Above,
        });
        m
    }

    fn build_decompositions() -> HashMap<char, Decomposition> {
        let mut m = HashMap::new();
        let entries: &[(char, char, &str)] = &[
            // Lowercase
            ('\u{e9}', 'e', "acute"),      // e-acute
            ('\u{e8}', 'e', "grave"),      // e-grave
            ('\u{ea}', 'e', "circumflex"), // e-circumflex
            ('\u{eb}', 'e', "diaeresis"),  // e-diaeresis
            ('\u{e0}', 'a', "grave"),      // a-grave
            ('\u{e2}', 'a', "circumflex"), // a-circumflex
            ('\u{f9}', 'u', "grave"),      // u-grave
            ('\u{fb}', 'u', "circumflex"), // u-circumflex
            ('\u{fc}', 'u', "diaeresis"),  // u-diaeresis
            ('\u{f4}', 'o', "circumflex"), // o-circumflex
            ('\u{f6}', 'o', "diaeresis"),  // o-diaeresis
            ('\u{ee}', 'i', "circumflex"), // i-circumflex
            ('\u{ef}', 'i', "diaeresis"),  // i-diaeresis
            ('\u{e7}', 'c', "cedilla"),    // c-cedilla
            ('\u{f1}', 'n', "tilde"),      // n-tilde
            // Uppercase
            ('\u{c9}', 'E', "acute"),      // E-acute
            ('\u{c8}', 'E', "grave"),      // E-grave
            ('\u{ca}', 'E', "circumflex"), // E-circumflex
            ('\u{cb}', 'E', "diaeresis"),  // E-diaeresis
            ('\u{c0}', 'A', "grave"),      // A-grave
            ('\u{c2}', 'A', "circumflex"), // A-circumflex
            ('\u{d9}', 'U', "grave"),      // U-grave
            ('\u{db}', 'U', "circumflex"), // U-circumflex
            ('\u{dc}', 'U', "diaeresis"),  // U-diaeresis
            ('\u{d4}', 'O', "circumflex"), // O-circumflex
            ('\u{d6}', 'O', "diaeresis"),  // O-diaeresis
            ('\u{ce}', 'I', "circumflex"), // I-circumflex
            ('\u{cf}', 'I', "diaeresis"),  // I-diaeresis
            ('\u{c7}', 'C', "cedilla"),    // C-cedilla
            ('\u{d1}', 'N', "tilde"),      // N-tilde
        ];
        for &(accented, base, accent_name) in entries {
            m.insert(accented, Decomposition { base, accent_name });
        }
        m
    }

    /// Pre-compose all accented characters in the decomposition table.
    fn precompose_all(&mut self) {
        let decomps: Vec<(char, char, &'static str)> = self
            .decompositions
            .iter()
            .map(|(&ch, d)| (ch, d.base, d.accent_name))
            .collect();

        for (accented, base, accent_name) in decomps {
            let base_shape = shape_char(base);
            if base_shape.is_empty() {
                continue;
            }
            if let Some(accent_def) = self.accents.get(accent_name) {
                let mut combined = base_shape;
                // For uppercase accents above, shift accent up slightly to clear the letter.
                // Uppercase letters occupy y=[0.0, 1.0], accents sit in [0.75, 1.0] zone.
                let is_uppercase = base.is_uppercase();
                let accent_shape: Vec<(f64, f64)> = if is_uppercase && accent_def.placement == AccentPlacement::Above {
                    accent_def.shape.iter().map(|&(x, y)| (x, y + 0.05)).collect()
                } else {
                    accent_def.shape.clone()
                };
                combined.extend(accent_shape);
                self.composed.insert(accented, combined);
            }
        }
    }
}

/// Return the polygon for a given character.
/// Uppercase A-Z, digits 0-9, and punctuation from the original shape_char.
/// Lowercase a-z added as new definitions.
/// Returns empty vec for truly unknown characters (caller uses fallback).
pub fn shape_char(c: char) -> Vec<(f64, f64)> {
    match c {
        // ================================================================
        // Digits 0-9 (identical to original hud.rs definitions)
        // ================================================================
        '0' => vec![
            (0.25, 0.), (0.75, 0.), (1., 0.2), (1., 0.8),
            (0.75, 1.), (0.25, 1.), (0., 0.8), (0., 0.2),
            (0.25, 0.2), (0.75, 0.6), (0.75, 0.8),
            (0.25, 0.375), (0.25, 0.8), (0.75, 0.8),
            (0.75, 0.2), (0., 0.2),
        ],
        '1' => vec![
            (0.125, 0.), (0.875, 0.), (0.875, 0.2),
            (0.625, 0.2), (0.625, 1.), (0.375, 1.),
            (0., 0.75), (0.15, 0.65), (0.375, 0.8),
            (0.375, 0.2), (0.125, 0.2),
        ],
        '2' => vec![
            (0., 0.), (1., 0.), (1., 0.2), (0.35, 0.2),
            (1., 0.5), (1., 0.8), (0.75, 1.), (0.25, 1.),
            (0., 0.8), (0., 0.6), (0.25, 0.6), (0.25, 0.8),
            (0.75, 0.8), (0.75, 0.6), (0., 0.2),
        ],
        '3' => vec![
            (0.25, 0.), (0.75, 0.), (1., 0.2), (1., 0.4),
            (0.875, 0.5), (1., 0.6), (1., 0.8), (0.75, 1.),
            (0.25, 1.), (0., 0.8), (0., 0.6), (0.25, 0.6),
            (0.25, 0.8), (0.75, 0.8), (0.75, 0.6), (0.5, 0.6),
            (0.5, 0.4), (0.75, 0.4), (0.75, 0.2), (0.25, 0.2),
            (0.25, 0.4), (0., 0.4), (0., 0.2),
        ],
        '4' => vec![
            (0.5, 0.), (0.75, 0.), (0.75, 1.), (0.5, 1.),
            (0., 0.4), (0., 0.2), (1., 0.2), (1., 0.4),
            (0.25, 0.4), (0.5, 0.8),
        ],
        '5' => vec![
            (0.25, 0.), (0.75, 0.), (1., 0.2), (1., 0.5),
            (0.25, 0.7), (0.25, 0.8), (1., 0.8), (1., 1.),
            (0., 1.), (0., 0.6), (0.75, 0.4), (0.75, 0.2),
            (0.25, 0.2), (0.25, 0.35), (0., 0.4), (0., 0.2),
            (0.25, 0.),
        ],
        '6' => vec![
            (0.25, 0.), (0.75, 0.), (1., 0.2), (1., 0.4),
            (0.75, 0.6), (0.25, 0.6), (0.25, 0.8), (1., 0.8),
            (0.75, 1.), (0.25, 1.), (0., 0.8), (0., 0.4),
            (0.75, 0.4), (0.75, 0.2), (0.25, 0.2), (0.25, 0.4),
            (0., 0.4), (0., 0.2),
        ],
        '7' => vec![
            (0.25, 0.), (0.5, 0.), (1., 0.8), (1., 1.),
            (0., 1.), (0., 0.8), (0.75, 0.8),
        ],
        '8' => vec![
            (0.25, 0.), (0.75, 0.), (1., 0.2), (1., 0.4),
            (0.875, 0.5), (1., 0.6), (1., 0.8), (0.75, 1.),
            (0.25, 1.), (0.25, 0.8), (0.75, 0.8), (0.75, 0.6),
            (0.25, 0.6), (0.25, 0.4), (0.75, 0.4), (0.75, 0.2),
            (0.25, 0.2), (0.25, 1.), (0., 0.8), (0., 0.6),
            (0.125, 0.5), (0., 0.4), (0., 0.2),
        ],
        '9' => vec![
            (0.75, 1.), (0.25, 1.), (0., 0.8), (0., 0.6),
            (0.25, 0.4), (0.75, 0.4), (0.75, 0.2), (0., 0.2),
            (0.25, 0.), (0.75, 0.), (1., 0.2), (1., 0.6),
            (0.25, 0.6), (0.25, 0.8), (0.75, 0.8), (0.75, 0.6),
            (1., 0.6), (1., 0.8),
        ],
        // ================================================================
        // Uppercase A-Z (identical to original hud.rs definitions)
        // ================================================================
        ' ' => vec![(0., 0.), (0., 0.), (0., 0.)],
        'A' => vec![
            (0., 0.), (0.25, 0.), (0.25, 0.4), (0.75, 0.4),
            (0.75, 0.4), (0.75, 0.6), (0.25, 0.6), (0.25, 0.8),
            (0.75, 0.8), (0.75, 0.), (1., 0.), (1., 0.8),
            (0.75, 1.), (0.25, 1.), (0., 0.8),
        ],
        'B' => vec![
            (0., 0.), (0.75, 0.), (1., 0.2), (1., 0.4),
            (0.875, 0.5), (1., 0.6), (1., 0.8), (0.75, 1.),
            (0.25, 1.), (0.25, 0.8), (0.75, 0.8), (0.75, 0.6),
            (0.25, 0.6), (0.25, 0.4), (0.75, 0.4), (0.75, 0.2),
            (0.25, 0.2), (0., 1.),
        ],
        'C' => vec![
            (0.25, 0.), (0.75, 0.), (1., 0.2), (1., 0.4),
            (0.75, 0.4), (0.75, 0.2), (0.25, 0.2), (0.25, 0.8),
            (0.75, 0.8), (0.75, 0.6), (1., 0.6), (1., 0.8),
            (0.75, 1.), (0.25, 1.), (0., 0.8), (0., 0.2),
        ],
        'D' => vec![
            (0., 0.), (0.75, 0.), (1., 0.2), (1., 0.8),
            (0.75, 1.), (0., 1.), (0., 0.2), (0.25, 0.2),
            (0.25, 0.8), (0.75, 0.8), (0.75, 0.2), (0., 0.2),
        ],
        'E' => vec![
            (0., 0.), (0.75, 0.), (1., 0.2), (0.25, 0.2),
            (0.25, 0.4), (0.5, 0.4), (0.5, 0.6), (0.25, 0.6),
            (0.25, 0.8), (1., 0.8), (0.75, 1.), (0., 1.),
        ],
        'F' => vec![
            (0., 0.), (0.25, 0.), (0.25, 0.4), (0.5, 0.4),
            (0.75, 0.6), (0.25, 0.6), (0.25, 0.8), (1., 0.8),
            (1., 1.), (0., 1.),
        ],
        'G' => vec![
            (0.25, 0.), (0.75, 0.), (1., 0.2), (1., 0.6),
            (0.5, 0.6), (0.5, 0.4), (0.75, 0.4), (0.75, 0.2),
            (0.25, 0.2), (0.25, 0.8), (1., 0.8), (0.75, 1.),
            (0.25, 1.), (0., 0.8), (0., 0.2),
        ],
        'H' => vec![
            (0., 1.), (0.25, 1.), (0.25, 0.6), (0.75, 0.6),
            (0.75, 1.), (1., 1.), (1., 0.), (0.75, 0.),
            (0.75, 0.4), (0.25, 0.4), (0.25, 0.), (0., 0.),
        ],
        'I' => vec![
            (0.125, 0.), (0.875, 0.), (0.875, 0.2), (0.625, 0.2),
            (0.625, 0.8), (0.875, 0.8), (0.875, 1.), (0.125, 1.),
            (0.125, 0.8), (0.375, 0.8), (0.375, 0.2), (0.125, 0.2),
        ],
        'J' => vec![
            (0.25, 1.), (0.5, 1.), (0.75, 0.8), (0.75, 0.2),
            (1., 0.2), (1., 0.), (0., 0.), (0., 0.2),
            (0.25, 0.2), (0.25, 0.8), (0., 0.8),
        ],
        'K' => vec![
            (0., 1.), (0.25, 1.), (0.25, 0.6), (0.75, 1.),
            (1., 1.), (0.375, 0.5), (1., 0.), (0.75, 0.),
            (0.25, 0.4), (0.25, 0.), (0., 0.),
        ],
        'L' => vec![
            (0., 0.), (0., 1.), (0.25, 1.), (0.25, 0.2),
            (1., 0.2), (1., 0.),
        ],
        'M' => vec![
            (0., 1.), (0.25, 1.), (0.5, 0.6), (0.75, 1.),
            (1., 1.), (1., 0.), (0.75, 0.), (0.75, 0.6),
            (0.5, 0.2), (0.25, 0.6), (0.25, 0.), (0., 0.),
        ],
        'N' => vec![
            (0., 1.), (0.25, 1.), (0.75, 0.4), (0.75, 1.),
            (1., 1.), (1., 0.), (0.75, 0.), (0.25, 0.6),
            (0.25, 0.), (0., 0.),
        ],
        'O' => vec![
            (0.25, 0.), (0.75, 0.), (1., 0.2), (1., 0.8),
            (0.75, 1.), (0.25, 1.), (0., 0.8), (0., 0.2),
            (0.25, 0.2), (0.25, 0.8), (0.75, 0.8), (0.75, 0.2),
            (0., 0.2),
        ],
        'P' => vec![
            (0., 0.), (0.25, 0.), (0.25, 0.5), (0.75, 0.5),
            (1., 0.6), (1., 0.8), (0.75, 1.), (0.25, 1.),
            (0., 1.),
        ],
        'Q' => vec![
            (0.25, 1.), (0.75, 1.), (1., 0.8), (1., 0.2),
            (0.75, 0.), (0.25, 0.), (0., 0.2), (0., 0.8),
            (0.25, 0.8), (0.25, 0.2), (0.75, 0.2), (0.75, 0.8),
            (0., 0.8), (0.5, 0.6), (1., 1.),
        ],
        'R' => vec![
            (0., 0.), (0.25, 0.), (0.25, 0.8), (0.75, 0.8),
            (0.75, 0.6), (0.25, 0.6), (0.25, 0.4), (0.75, 0.),
            (1., 0.), (0.5, 0.4), (0.75, 0.4), (1., 0.6),
            (1., 0.8), (0.75, 1.), (0., 1.),
        ],
        'S' => vec![
            (0.25, 0.), (0.75, 0.), (1., 0.2), (1., 0.4),
            (0.75, 0.6), (0.25, 0.6), (0.25, 0.8), (1., 0.8),
            (0.75, 1.), (0.25, 1.), (0., 0.8), (0., 0.6),
            (0.25, 0.4), (0.75, 0.4), (0.75, 0.2), (0., 0.2),
        ],
        'T' => vec![
            (0.385, 0.), (0.625, 0.), (0.625, 0.8), (1., 0.8),
            (1., 1.), (0., 1.), (0., 0.8), (0.385, 0.8),
        ],
        'U' => vec![
            (0., 1.), (0.25, 1.), (0.25, 0.2), (0.75, 0.2),
            (0.75, 1.), (1., 1.), (1., 0.), (0.75, 0.),
            (0.25, 0.), (0., 0.),
        ],
        'V' => vec![
            (0., 1.), (0.25, 1.), (0.5, 0.2), (0.75, 1.),
            (1., 1.), (0.6, 0.), (0.4, 0.),
        ],
        'W' => vec![
            (0., 1.), (0.2, 0.), (0.4, 0.), (0.5, 0.2),
            (0.6, 0.), (0.8, 0.), (1., 1.), (0.6, 0.4),
            (0.6, 0.6), (0.4, 0.6), (0.4, 0.4), (0.2, 1.),
        ],
        'X' => vec![
            (0., 1.), (0.25, 1.), (0.5, 0.6), (0.75, 1.),
            (1., 1.), (0.625, 0.5), (1., 0.), (0.75, 0.),
            (0.5, 0.4), (0.25, 0.), (0., 0.), (0.375, 0.5),
        ],
        'Y' => vec![
            (0., 0.), (0.25, 0.), (0.5, 0.4), (0.75, 0.),
            (1., 0.), (0.625, 0.6), (0.625, 1.), (0.375, 1.),
            (0.375, 0.6),
        ],
        'Z' => vec![
            (0., 1.), (1., 1.), (1., 0.8), (0.25, 0.2),
            (1., 0.2), (0.75, 0.), (0., 0.), (0., 0.2),
            (0.75, 0.8), (0., 0.8),
        ],
        // ================================================================
        // Punctuation (original + new for French)
        // ================================================================
        ':' => vec![
            (0.3, 0.8), (0.7, 0.8), (0.7, 0.6), (0.3, 0.6),
            (0.3, 0.4), (0.7, 0.4), (0.7, 0.2), (0.3, 0.2),
            (0.3, 0.4), (0.7, 0.4),
        ],
        '-' => vec![(0.1, 0.6), (0.9, 0.6), (0.9, 0.4), (0.1, 0.4)],
        '.' => vec![(0.3, 1.), (0.7, 1.), (0.7, 0.8), (0.3, 0.8)],
        '!' => vec![
            (0.35, 1.), (0.65, 1.), (0.65, 0.8), (0.35, 0.8),
            (0.35, 0.65), (0.65, 0.65), (0.65, 0.),
            (0.35, 0.),
        ],
        '?' => vec![
            (0.35, 0.0), (0.65, 0.0), (0.65, 0.15), (0.35, 0.15),
            (0.35, 0.25), (0.65, 0.25), (0.65, 0.45),
            (0.85, 0.55), (0.85, 0.85), (0.65, 1.0),
            (0.35, 1.0), (0.15, 0.85), (0.15, 0.65),
            (0.35, 0.65), (0.35, 0.8), (0.65, 0.8),
            (0.65, 0.65), (0.35, 0.45),
        ],
        '\'' => vec![
            (0.35, 0.7), (0.65, 0.7), (0.65, 1.0), (0.35, 1.0),
        ],
        // Guillemet left: <<
        '\u{ab}' => vec![
            (0.0, 0.5), (0.45, 0.8), (0.45, 0.65), (0.15, 0.5),
            (0.45, 0.35), (0.45, 0.2),
            (0.55, 0.5), (1.0, 0.8), (1.0, 0.65), (0.7, 0.5),
            (1.0, 0.35), (1.0, 0.2),
        ],
        // Guillemet right: >>
        '\u{bb}' => vec![
            (0.0, 0.2), (0.0, 0.35), (0.3, 0.5), (0.0, 0.65),
            (0.0, 0.8), (0.45, 0.5),
            (0.55, 0.2), (0.55, 0.35), (0.85, 0.5), (0.55, 0.65),
            (0.55, 0.8), (1.0, 0.5),
        ],
        // ================================================================
        // Lowercase a-z (x-height: y in [0.0, 0.65])
        // Ascenders extend to y=1.0, descenders to y=-0.2
        // ================================================================
        'a' => vec![
            (0.75, 0.0), (0.75, 0.65), (0.55, 0.65), (0.2, 0.55),
            (0.0, 0.35), (0.0, 0.15), (0.2, 0.0), (0.55, 0.0),
            (0.75, 0.15), (0.55, 0.15), (0.55, 0.0), (0.2, 0.0),
            (0.2, 0.15), (0.55, 0.15), (0.55, 0.5),
            (0.2, 0.5), (0.2, 0.35), (0.55, 0.35),
        ],
        'b' => vec![
            (0.0, 0.0), (0.0, 1.0), (0.2, 1.0), (0.2, 0.55),
            (0.55, 0.65), (0.8, 0.55), (1.0, 0.35),
            (1.0, 0.15), (0.8, 0.0), (0.2, 0.0),
            (0.2, 0.15), (0.75, 0.15), (0.75, 0.5),
            (0.2, 0.5), (0.2, 0.15),
        ],
        'c' => vec![
            (0.8, 0.0), (0.2, 0.0), (0.0, 0.15), (0.0, 0.5),
            (0.2, 0.65), (0.8, 0.65), (0.8, 0.5),
            (0.2, 0.5), (0.2, 0.15), (0.8, 0.15),
        ],
        'd' => vec![
            (1.0, 0.0), (1.0, 1.0), (0.8, 1.0), (0.8, 0.55),
            (0.45, 0.65), (0.2, 0.55), (0.0, 0.35),
            (0.0, 0.15), (0.2, 0.0), (0.8, 0.0),
            (0.8, 0.15), (0.25, 0.15), (0.25, 0.5),
            (0.8, 0.5), (0.8, 0.15),
        ],
        'e' => vec![
            (0.0, 0.35), (0.8, 0.35), (0.8, 0.5), (0.2, 0.5),
            (0.2, 0.65), (0.8, 0.65), (0.8, 0.5),
            (1.0, 0.35), (1.0, 0.15), (0.8, 0.0),
            (0.2, 0.0), (0.0, 0.15),
        ],
        'f' => vec![
            (0.2, 0.0), (0.2, 0.5), (0.0, 0.5), (0.0, 0.65),
            (0.2, 0.65), (0.2, 0.85), (0.4, 1.0), (0.7, 1.0),
            (0.7, 0.85), (0.45, 0.85), (0.4, 0.65),
            (0.6, 0.65), (0.6, 0.5), (0.4, 0.5), (0.4, 0.0),
        ],
        'g' => vec![
            (0.8, 0.65), (0.2, 0.65), (0.0, 0.5), (0.0, 0.15),
            (0.2, 0.0), (0.8, 0.0), (0.8, -0.05),
            (0.8, -0.15), (0.6, -0.2), (0.2, -0.2),
            (0.2, -0.05), (0.6, -0.05), (0.6, 0.0),
            (0.2, 0.0), (0.2, 0.15), (0.6, 0.15),
            (0.6, 0.5), (0.2, 0.5), (0.2, 0.15),
            (0.6, 0.15), (0.6, 0.5), (0.8, 0.5),
        ],
        'h' => vec![
            (0.0, 0.0), (0.0, 1.0), (0.2, 1.0), (0.2, 0.55),
            (0.55, 0.65), (0.8, 0.55), (0.8, 0.0),
            (0.6, 0.0), (0.6, 0.5), (0.2, 0.5),
            (0.2, 0.0),
        ],
        'i' => vec![
            (0.3, 0.0), (0.7, 0.0), (0.7, 0.65), (0.3, 0.65),
            (0.3, 0.75), (0.7, 0.75), (0.7, 0.9),
            (0.3, 0.9),
        ],
        'j' => vec![
            (0.4, 0.65), (0.7, 0.65), (0.7, -0.05), (0.5, -0.2),
            (0.2, -0.2), (0.2, -0.05), (0.5, -0.05),
            (0.5, 0.65), (0.4, 0.75), (0.7, 0.75),
            (0.7, 0.9), (0.4, 0.9),
        ],
        'k' => vec![
            (0.0, 0.0), (0.0, 1.0), (0.2, 1.0), (0.2, 0.4),
            (0.6, 0.65), (0.85, 0.65), (0.3, 0.3),
            (0.85, 0.0), (0.6, 0.0), (0.2, 0.2),
            (0.2, 0.0),
        ],
        'l' => vec![
            (0.3, 0.0), (0.3, 1.0), (0.55, 1.0), (0.55, 0.15),
            (0.7, 0.0),
        ],
        'm' => vec![
            (0.0, 0.0), (0.0, 0.65), (0.2, 0.65), (0.2, 0.15),
            (0.4, 0.15), (0.5, 0.55), (0.6, 0.15),
            (0.8, 0.15), (0.8, 0.65), (1.0, 0.65),
            (1.0, 0.0), (0.8, 0.0), (0.8, 0.15),
            (0.6, 0.15), (0.5, 0.0), (0.4, 0.15),
            (0.2, 0.15), (0.2, 0.0),
        ],
        'n' => vec![
            (0.0, 0.0), (0.0, 0.65), (0.2, 0.65), (0.2, 0.15),
            (0.6, 0.5), (0.6, 0.65), (0.8, 0.65),
            (0.8, 0.0), (0.6, 0.0), (0.6, 0.5),
            (0.2, 0.15), (0.2, 0.0),
        ],
        'o' => vec![
            (0.2, 0.0), (0.0, 0.15), (0.0, 0.5), (0.2, 0.65),
            (0.8, 0.65), (1.0, 0.5), (1.0, 0.15),
            (0.8, 0.0), (0.2, 0.15), (0.2, 0.5),
            (0.8, 0.5), (0.8, 0.15), (0.2, 0.15),
        ],
        'p' => vec![
            (0.0, -0.2), (0.0, 0.65), (0.55, 0.65), (0.8, 0.55),
            (1.0, 0.35), (1.0, 0.15), (0.8, 0.0),
            (0.2, 0.0), (0.2, 0.15), (0.75, 0.15),
            (0.75, 0.5), (0.2, 0.5), (0.2, -0.2),
        ],
        'q' => vec![
            (1.0, -0.2), (0.8, -0.2), (0.8, 0.0), (0.45, 0.0),
            (0.2, 0.1), (0.0, 0.3), (0.0, 0.5),
            (0.2, 0.65), (0.8, 0.65), (0.8, 0.5),
            (0.25, 0.5), (0.25, 0.15), (0.8, 0.15),
            (1.0, 0.0),
        ],
        'r' => vec![
            (0.0, 0.0), (0.0, 0.65), (0.2, 0.65), (0.2, 0.5),
            (0.55, 0.65), (0.8, 0.65), (0.8, 0.5),
            (0.4, 0.5), (0.2, 0.4), (0.2, 0.0),
        ],
        's' => vec![
            (0.0, 0.0), (0.6, 0.0), (0.8, 0.1), (0.8, 0.3),
            (0.6, 0.4), (0.2, 0.4), (0.2, 0.5),
            (0.8, 0.5), (0.8, 0.65), (0.2, 0.65),
            (0.0, 0.55), (0.0, 0.35), (0.2, 0.25),
            (0.6, 0.25), (0.6, 0.15), (0.0, 0.15),
        ],
        't' => vec![
            (0.2, 0.0), (0.2, 0.5), (0.0, 0.5), (0.0, 0.65),
            (0.2, 0.65), (0.2, 1.0), (0.4, 1.0),
            (0.4, 0.65), (0.7, 0.65), (0.7, 0.5),
            (0.4, 0.5), (0.4, 0.15), (0.6, 0.0),
        ],
        'u' => vec![
            (0.0, 0.65), (0.2, 0.65), (0.2, 0.15), (0.6, 0.15),
            (0.6, 0.65), (0.8, 0.65), (0.8, 0.0),
            (0.2, 0.0), (0.0, 0.15),
        ],
        'v' => vec![
            (0.0, 0.65), (0.2, 0.65), (0.5, 0.1), (0.8, 0.65),
            (1.0, 0.65), (0.6, 0.0), (0.4, 0.0),
        ],
        'w' => vec![
            (0.0, 0.65), (0.15, 0.0), (0.3, 0.0), (0.5, 0.4),
            (0.7, 0.0), (0.85, 0.0), (1.0, 0.65),
            (0.8, 0.65), (0.7, 0.3), (0.5, 0.55),
            (0.3, 0.3), (0.2, 0.65),
        ],
        'x' => vec![
            (0.0, 0.65), (0.2, 0.65), (0.5, 0.4), (0.8, 0.65),
            (1.0, 0.65), (0.6, 0.325), (1.0, 0.0),
            (0.8, 0.0), (0.5, 0.25), (0.2, 0.0),
            (0.0, 0.0), (0.4, 0.325),
        ],
        'y' => vec![
            (0.0, 0.65), (0.2, 0.65), (0.5, 0.15), (0.8, 0.65),
            (1.0, 0.65), (0.6, -0.05), (0.5, -0.2),
            (0.2, -0.2), (0.2, -0.05), (0.4, -0.05),
        ],
        'z' => vec![
            (0.0, 0.65), (0.8, 0.65), (0.8, 0.5), (0.2, 0.15),
            (0.8, 0.15), (0.8, 0.0), (0.0, 0.0),
            (0.0, 0.15), (0.6, 0.5), (0.0, 0.5),
        ],
        _ => vec![],
    }
}
```

### Step 4.2: Export module in `lib.rs`

- [ ] In `src/lib.rs`, add `pub mod glyphs;` after the existing module declarations.

### Step 4.3: Update `hud.rs` to use `glyphs::glyph()`

- [ ] In `src/rendering/hud.rs`:
  - Remove the entire `fn shape_char(c: char) -> Vec<(f64, f64)>` function (approximately lines 14-276).
  - In `render_char`, change `let shape = shape_char(c);` to `let shape = crate::glyphs::shape_char(c);` (temporary -- will use GlyphCache in Step 4.6).
  - In `render_string`, remove the line `let c = c.to_ascii_uppercase();`.

### Step 4.4: Verify compilation and run tests

- [ ] Run:

```bash
rtk cargo check && rtk cargo test
```

**Expected:** Clean compilation, all existing tests pass. Uppercase text still renders via `shape_char` match arms, now in `glyphs.rs`.

### Step 4.5: Add `GlyphCache` to `Globals`

- [ ] In `src/parameters.rs`, add import:

```rust
use crate::glyphs::GlyphCache;
```

- [ ] Add field to `Globals` (after `locale`):

```rust
pub glyph_cache: GlyphCache,
```

- [ ] Initialize in `Globals::new()`:

```rust
glyph_cache: GlyphCache::new(),
```

### Step 4.6: Update `render_char` to use `GlyphCache`

- [ ] In `src/rendering/hud.rs`, update `render_char` signature to accept the glyph cache:

```rust
fn render_char(
    encadrement: &[(f64, f64); 4],
    c: char,
    color: [f32; 4],
    renderer: &mut Renderer2D,
    render_scale: f64,
    glyph_cache: &crate::glyphs::GlyphCache,
) {
    let shape = glyph_cache.glyph(c);
    let pts = displace_shape(encadrement, &shape, render_scale);
    renderer.hud_fill_poly(&pts, color);
}
```

- [ ] Update `render_string` to accept and pass through `GlyphCache`:
  - Add `glyph_cache: &crate::glyphs::GlyphCache` parameter after `globals`.
  - Pass it to `render_char`.

- [ ] Update all call sites of `render_string`:
  - In `render_hud` (in `hud.rs`): extract `let glyph_cache = &globals.glyph_cache;` and pass to each `render_string` call.
  - In `apply_button` (in `pause_menu.rs`): pass `&globals.glyph_cache` to each `render_string` call.
  - In `render_button_tooltip` (in `pause_menu.rs`): pass `&globals.glyph_cache` to each `render_string` call.
  - In `render_pause_title` (in `pause_menu.rs`): pass `&globals.glyph_cache` to each `render_string` call.

**Note on borrow checker:** `render_string` takes `globals: &Globals` (shared ref). `glyph_cache` is a separate `&GlyphCache` param extracted before the call, so no borrow conflict.

### Step 4.7: Verify compilation and run tests

- [ ] Run:

```bash
rtk cargo check && rtk cargo test
```

**Expected:** Clean compilation, all tests pass.

### Step 4.8: Add glyph unit tests

- [ ] In `tests/locale_tests.rs`, add:

```rust
use asteroids::glyphs::{GlyphCache, shape_char};

#[test]
fn test_shape_char_uppercase_exists() {
    for c in 'A'..='Z' {
        let shape = shape_char(c);
        assert!(!shape.is_empty(), "Missing uppercase glyph for '{}'", c);
    }
}

#[test]
fn test_shape_char_digits_exist() {
    for c in '0'..='9' {
        let shape = shape_char(c);
        assert!(!shape.is_empty(), "Missing digit glyph for '{}'", c);
    }
}

#[test]
fn test_shape_char_lowercase_exists() {
    for c in 'a'..='z' {
        let shape = shape_char(c);
        assert!(!shape.is_empty(), "Missing lowercase glyph for '{}'", c);
    }
}

#[test]
fn test_shape_char_punctuation() {
    for c in [':', '-', '.', '!', '?', '\''] {
        let shape = shape_char(c);
        assert!(!shape.is_empty(), "Missing punctuation glyph for '{}'", c);
    }
}

#[test]
fn test_glyph_cache_composed_accents() {
    let cache = GlyphCache::new();
    // French accented chars should all be present via composition
    let accented = [
        '\u{e9}', '\u{e8}', '\u{ea}', '\u{eb}', // e-accents
        '\u{e0}', '\u{e2}',                       // a-accents
        '\u{f9}', '\u{fb}', '\u{fc}',             // u-accents
        '\u{f4}', '\u{ee}', '\u{ef}',             // o/i-accents
        '\u{e7}', '\u{f1}',                       // cedilla, tilde
    ];
    for c in accented {
        let shape = cache.glyph(c);
        assert!(shape.len() > 3, "Composed glyph for '{}' should have >3 points, got {}", c, shape.len());
    }
}

#[test]
fn test_glyph_cache_uppercase_accents() {
    let cache = GlyphCache::new();
    let accented = [
        '\u{c9}', '\u{c8}', '\u{ca}', '\u{cb}', // E-accents
        '\u{c0}', '\u{c2}',                       // A-accents
        '\u{c7}',                                  // C-cedilla
    ];
    for c in accented {
        let shape = cache.glyph(c);
        assert!(shape.len() > 3, "Composed uppercase glyph for '{}' should have >3 points, got {}", c, shape.len());
    }
}

#[test]
fn test_glyph_cache_fallback_square() {
    let cache = GlyphCache::new();
    // Unknown char should produce a filled square (4 points)
    let shape = cache.glyph('\u{2603}'); // snowman
    assert_eq!(shape.len(), 4);
}

#[test]
fn test_glyph_cache_guillemets() {
    let cache = GlyphCache::new();
    let left = cache.glyph('\u{ab}');
    let right = cache.glyph('\u{bb}');
    assert!(!left.is_empty(), "Missing left guillemet glyph");
    assert!(!right.is_empty(), "Missing right guillemet glyph");
}
```

- [ ] Run:

```bash
rtk cargo test --test locale_tests
```

**Expected:** All glyph tests pass.

### Step 4.9: Commit

```bash
rtk git add src/glyphs.rs src/lib.rs src/rendering/hud.rs src/parameters.rs src/pause_menu.rs tests/locale_tests.rs && rtk git commit -m "feat: glyphs module with three-tier lookup, lowercase a-z, accents, and composition"
```

---

## Task 5: Replace hardcoded HUD strings with locale lookups

**Goal:** All user-visible text in `render_hud` uses `globals.locale.get()` instead of hardcoded strings.

### Step 5.1: Update score display

- [ ] In `src/rendering/hud.rs`, in `render_hud`, change:

```rust
let score_str = format!("SCORE {}", state.score);
```

to:

```rust
let score_str = format!("{} {}", globals.locale.get("score_label"), state.score);
```

### Step 5.2: Update stage display

- [ ] Change:

```rust
let stage_str = format!("STAGE {}", state.stage);
```

to:

```rust
let stage_str = format!("{} {}", globals.locale.get("stage_label"), state.stage);
```

### Step 5.3: Update teleport ready indicator

- [ ] Change the hardcoded `'F'` character in the teleport ready section:

```rust
let tp_char = globals.locale.get("teleport_ready").chars().next().unwrap_or('F');
render_char(
    &encadrement,
    tp_char,
    cyan,
    renderer,
    globals.render.render_scale,
    &globals.glyph_cache,
);
```

### Step 5.4: Update debug stats labels

- [ ] Change the `debug_lines` array to use locale keys:

```rust
let debug_lines = [
    format!("{:<11}: {}", globals.locale.get("debug_fps"), fps),
    format!("{:<11}: {}", globals.locale.get("debug_peak_fps"), peak_fps),
    format!("{:<11}: {}", globals.locale.get("debug_objects"), nb_objets),
    format!("{:<11}: {}", globals.locale.get("debug_toosmall"), nb_toosmall),
    format!("{:<11}: {}", globals.locale.get("debug_frags"), nb_frags),
    format!("{:<11}: {}", globals.locale.get("debug_projectiles"), nb_projs),
    format!("{:<11}: {}", globals.locale.get("debug_explosions"), nb_explos),
    format!("{:<11}: {}", globals.locale.get("debug_smoke"), nb_smoke),
    format!("{:<11}: {}", globals.locale.get("debug_chunks"), nb_chunks),
    format!("{:<11}: {}", globals.locale.get("debug_chunks_explo"), nb_chunks_e),
];
```

### Step 5.5: Verify compilation

- [ ] Run:

```bash
rtk cargo check && rtk cargo test
```

**Expected:** Clean compilation, all tests pass.

### Step 5.6: Commit

```bash
rtk git add src/rendering/hud.rs && rtk git commit -m "feat: HUD text uses locale key lookups instead of hardcoded strings"
```

---

## Task 6: Replace hardcoded pause menu strings with locale lookups

**Goal:** Pause menu button labels and tooltips use locale key resolution at render time.

### Step 6.1: Change `ButtonBoolean` text fields from `&'static str` to locale keys

- [ ] In `src/pause_menu.rs`, change the `ButtonBoolean` struct:

```rust
pub struct ButtonBoolean {
    pub pos1: (f64, f64),
    pub pos2: (f64, f64),
    /// Locale key for the button label.
    pub text_key: &'static str,
    /// Locale key for the tooltip.
    pub tooltip_key: &'static str,
    pub field: GlobalToggle,
    pub last_mouse_state: bool,
}
```

### Step 6.2: Update `make_buttons` to use locale keys

- [ ] Update the macro and all button definitions:

```rust
macro_rules! btn {
    ($text_key:expr, $tooltip_key:expr,
     $c1:expr, $r1:expr, $c2:expr, $r2:expr,
     $field:expr) => {
        ButtonBoolean {
            pos1: (sx + $c1 / 16.0 * w, sy + $r1 / 24.0 * h),
            pos2: (sx + $c2 / 16.0 * w, sy + $r2 / 24.0 * h),
            text_key: $text_key,
            tooltip_key: $tooltip_key,
            field: $field,
            last_mouse_state: false,
        }
    };
}
vec![
    btn!("btn_quit", "btn_quit_tip", 10.0, 20.0, 12.0, 22.0, GlobalToggle::Quit),
    btn!("btn_resume", "btn_resume_tip", 7.0, 20.0, 9.0, 22.0, GlobalToggle::Pause),
    btn!("btn_new_game", "btn_new_game_tip", 4.0, 20.0, 6.0, 22.0, GlobalToggle::Restart),
    btn!("btn_advanced_hitbox", "btn_advanced_hitbox_tip", 10.0, 9.0, 12.0, 11.0, GlobalToggle::AdvancedHitbox),
    btn!("btn_smoke", "btn_smoke_tip", 7.0, 6.0, 9.0, 8.0, GlobalToggle::Smoke),
    btn!("btn_screenshake", "btn_screenshake_tip", 4.0, 6.0, 6.0, 8.0, GlobalToggle::Screenshake),
    btn!("btn_flashes", "btn_flashes_tip", 10.0, 6.0, 12.0, 8.0, GlobalToggle::Flashes),
    btn!("btn_chunks", "btn_chunks_tip", 7.0, 3.0, 9.0, 5.0, GlobalToggle::Chunks),
    btn!("btn_color_effects", "btn_color_effects_tip", 10.0, 3.0, 12.0, 5.0, GlobalToggle::DynColor),
]
```

### Step 6.3: Update `apply_button` to resolve locale keys at render time

- [ ] In `apply_button`, resolve the text at render time:

```rust
let text = globals.locale.get(btn.text_key);
```

- [ ] Replace all references to `btn.text` with `text` (the resolved string).

- [ ] Update `text_total_w` calculation to use `text.len()`.

- [ ] Pass `text` to `render_string` calls instead of `btn.text`.

### Step 6.4: Update `render_button_tooltip` to resolve locale keys

- [ ] Resolve the tooltip at render time:

```rust
let tooltip = globals.locale.get(btn.tooltip_key);
```

- [ ] Pass `tooltip` to `render_string` calls instead of `btn.text_over`.

### Step 6.5: Update pause title to use locale

- [ ] In `render_pause_title`, change `"ASTEROIDS"` to `globals.locale.get("pause_title")`:

```rust
let title = globals.locale.get("pause_title");
// Shadow
render_string(
    title,
    // ... same position args ...
);
// White title
render_string(
    title,
    // ... same position args ...
);
```

### Step 6.6: Verify compilation and run tests

- [ ] Run:

```bash
rtk cargo check && rtk cargo test
```

**Expected:** Clean compilation, all tests pass.

### Step 6.7: Commit

```bash
rtk git add src/pause_menu.rs && rtk git commit -m "feat: pause menu buttons use locale key resolution at render time"
```

---

## Task 7: Integration testing and final verification

**Goal:** Verify the full i18n pipeline works end-to-end, determinism is unaffected.

### Step 7.1: Add locale integration test

- [ ] In `tests/locale_tests.rs`, add:

```rust
#[test]
fn test_resolve_locale_english_default() {
    let locale = asteroids::locale::resolve_locale(None);
    // Without system locale override, should load English (or system locale)
    // Just verify it doesn't panic and has a valid code
    assert!(!locale.code.is_empty());
}

#[test]
fn test_resolve_locale_explicit_french() {
    let locale = asteroids::locale::resolve_locale(Some("fr"));
    assert_eq!(locale.code, "fr");
    assert_eq!(locale.get("stage_label"), "NIVEAU");
    // Fallback: keys present in both should resolve
    assert_eq!(locale.get("score_label"), "SCORE");
}

#[test]
fn test_resolve_locale_unknown_falls_back() {
    let locale = asteroids::locale::resolve_locale(Some("zz"));
    // Unknown language falls back to English
    assert_eq!(locale.code, "en");
}
```

### Step 7.2: Verify existing scenario tests still pass

- [ ] Run:

```bash
rtk cargo test --test scenario_tests
```

**Expected:** All scenario tests pass (locale is render-only, no physics impact).

### Step 7.3: Run full test suite

- [ ] Run:

```bash
rtk cargo test
```

**Expected:** All tests pass.

### Step 7.4: Run clippy

- [ ] Run:

```bash
rtk cargo clippy -- -W clippy::all
```

**Expected:** No new warnings.

### Step 7.5: Commit

```bash
rtk git add tests/locale_tests.rs && rtk git commit -m "test: integration tests for locale resolution and glyph system"
```

---

## Task 8: Update French locale with accented characters

**Goal:** Now that the glyph system supports accented characters, update `locales/fr.ron` to use proper French text with accents.

### Step 8.1: Update `locales/fr.ron` with accented strings

- [ ] Update `locales/fr.ron` -- replace ASCII approximations with proper French accented characters:

Key changes (showing only the lines that change):
```
"name": "Fran\u00e7ais"                   (was "Francais")
"btn_advanced_hitbox": "Hitbox avanc\u00e9e"     (was "Hitbox avancee")
"btn_advanced_hitbox_tip": "Une hitbox plus pr\u00e9cise."
"btn_smoke": "particules de fum\u00e9e"
"btn_smoke_tip": "Active la fum\u00e9e. D\u00e9sactiver pour de meilleures performances."
"btn_flashes_tip": "Active les flashs lumineux pour les \u00e9v\u00e9nements"
"btn_chunks": "particules de d\u00e9bris"
"btn_chunks_tip": "Active les d\u00e9bris. D\u00e9sactiver pour de meilleures performances."
```

**Note:** If RON does not support `\u` escapes in strings, use the literal UTF-8 characters directly. RON 0.8 supports UTF-8 natively in strings.

### Step 8.2: Verify French locale loads correctly with accents

- [ ] Add test in `tests/locale_tests.rs`:

```rust
#[test]
fn test_french_locale_accented_strings() {
    let locale = asteroids::locale::resolve_locale(Some("fr"));
    let smoke = locale.get("btn_smoke");
    // Should contain accented characters
    assert!(smoke.contains("fum"), "French smoke label should contain 'fum': got '{}'", smoke);
}
```

- [ ] Run:

```bash
rtk cargo test --test locale_tests
```

**Expected:** All tests pass.

### Step 8.3: Commit

```bash
rtk git add locales/fr.ron tests/locale_tests.rs && rtk git commit -m "feat: French locale with proper accented characters"
```

---

## Summary

| Task | Description | New/Modified Files | Tests |
|------|-------------|-------------------|-------|
| 1 | Locale module skeleton | `src/locale.rs`, `src/lib.rs`, `Cargo.toml` | Compile check |
| 2 | RON locale files + unit tests | `locales/en.ron`, `locales/fr.ron`, `tests/locale_tests.rs` | 4 locale unit tests |
| 3 | Integrate locale into Globals + CLI | `src/parameters.rs`, `src/main.rs` | Full suite |
| 4 | Glyph module with three-tier lookup | `src/glyphs.rs`, `src/lib.rs`, `src/rendering/hud.rs`, `src/parameters.rs`, `src/pause_menu.rs` | 9 glyph unit tests |
| 5 | HUD strings use locale lookups | `src/rendering/hud.rs` | Full suite |
| 6 | Pause menu uses locale keys | `src/pause_menu.rs` | Full suite |
| 7 | Integration tests + verification | `tests/locale_tests.rs` | 3 integration tests |
| 8 | French locale with accented chars | `locales/fr.ron`, `tests/locale_tests.rs` | Accent test |

**Total: 8 tasks, ~17 unit/integration tests, 8 commits.**

**Deferred (not in this plan):**
- Scope B: Additional Western European locales
- Scope C: Full Latin Extended glyph support
- Visual testing of glyph rendering (requires screen)
- Dark theme detection via OS settings
- MSDF font replacement (future phase)

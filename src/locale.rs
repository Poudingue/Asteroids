use std::collections::HashMap;
use std::path::Path;

pub struct Locale {
    pub strings: HashMap<String, String>,
    pub fallback: Option<Box<Locale>>,
}

impl Locale {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read locale file {:?}: {}", path, e))?;
        let strings: HashMap<String, String> = ron::from_str(&content)
            .map_err(|e| format!("Failed to parse locale file {:?}: {}", path, e))?;
        Ok(Self {
            strings,
            fallback: None,
        })
    }

    pub fn with_fallback(mut self, fallback: Locale) -> Self {
        self.fallback = Some(Box::new(fallback));
        self
    }

    pub fn get<'s, 'k: 's>(&'s self, key: &'k str) -> &'s str {
        if let Some(value) = self.strings.get(key) {
            value
        } else if let Some(ref fallback) = self.fallback {
            fallback.get(key)
        } else {
            key
        }
    }
}

pub fn detect_system_locale() -> String {
    "en".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detect_locale_returns_en() {
        assert_eq!(detect_system_locale(), "en");
    }

    #[test]
    fn locale_get_returns_value() {
        let mut strings = HashMap::new();
        strings.insert("greeting".to_string(), "Hello".to_string());
        let locale = Locale {
            strings,
            fallback: None,
        };
        assert_eq!(locale.get("greeting"), "Hello");
    }

    #[test]
    fn locale_get_returns_key_on_miss() {
        let locale = Locale {
            strings: HashMap::new(),
            fallback: None,
        };
        assert_eq!(locale.get("missing_key"), "missing_key");
    }

    #[test]
    fn locale_fallback_chain() {
        let mut en_strings = HashMap::new();
        en_strings.insert("greeting".to_string(), "Hello".to_string());
        en_strings.insert("farewell".to_string(), "Goodbye".to_string());
        let en = Locale {
            strings: en_strings,
            fallback: None,
        };

        let mut fr_strings = HashMap::new();
        fr_strings.insert("greeting".to_string(), "Bonjour".to_string());
        let fr = Locale {
            strings: fr_strings,
            fallback: None,
        }
        .with_fallback(en);

        assert_eq!(fr.get("greeting"), "Bonjour");
        assert_eq!(fr.get("farewell"), "Goodbye");
        assert_eq!(fr.get("unknown"), "unknown");
    }

    #[test]
    fn locale_load_from_ron_file() {
        let dir = std::env::temp_dir().join("claude_test_locale");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ron");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, r#"{{"greeting": "Hi", "quit": "Exit"}}"#).unwrap();

        let locale = Locale::load(&path).unwrap();
        assert_eq!(locale.get("greeting"), "Hi");
        assert_eq!(locale.get("quit"), "Exit");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_english_locale() {
        let locale = Locale::load(std::path::Path::new("locales/en.ron")).unwrap();
        assert_eq!(locale.get("pause_title"), "PAUSED");
    }
}

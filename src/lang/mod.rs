use std::collections::HashMap;
use serde::{Serialize, Deserialize};

mod english;
mod german;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Language {
    English,
    German,
}

pub struct TextDatabase {
    pub current_language: Language,
    translations: HashMap<Language, Translation>,
}

impl Default for TextDatabase {
    fn default() -> Self {
        let mut translations = HashMap::new();
        translations.insert(Language::English, english::new());
        translations.insert(Language::German, german::new());
        
        Self {
            current_language: Language::English,
            translations,
        }
    }
}

impl TextDatabase {
    pub fn translation(&self) -> &Translation {
        self.translations.get(&self.current_language).unwrap() // Never panics
    }
}

type Text = &'static str;

#[derive(Default, Clone)]
pub struct Translation {
    pub update_screen_install_label: Text,
    pub update_screen_update_label: Text,
    pub context_menu_paste: Text,
}

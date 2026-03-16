use std::collections::HashMap;

mod english;
mod german;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum Language {
    English,
    German,
}

pub struct TextDatabase {
    translations: HashMap<Language, Translation>,
}

impl Default for TextDatabase {
    fn default() -> Self {
        let mut translations = HashMap::new();
        translations.insert(Language::English, english::new());
        translations.insert(Language::German, german::new());
        
        Self {
            translations,
        }
    }
}

impl TextDatabase {
    pub fn get_translation(&self, language: Language) -> &Translation {
        self.translations.get(&language).unwrap()
    }
}

type Text = &'static str;

#[derive(Default, Clone)]
pub struct Translation {
    pub paste: Text,
    pub link_text_input_description: Text,
}

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

mod english;
mod german;

#[derive(Default, Hash, Eq, PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Language {
    #[default]
    English,
    German,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::English => "English (en)",
            Self::German => "Deutsch (de)",
        };

        write!(f, "{s}")
    }
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
    pub general_yes: Text,
    pub general_no: Text,
    pub general_close: Text,
    
    pub update_screen_caption_install: Text,
    pub update_screen_caption_update: Text,

    pub home_screen_update_popup_caption: Text,
    pub home_screen_update_popup_button_update_now: Text,
    pub home_screen_update_pupup_button_update_later: Text,

    pub home_screen_menu_settings: Text,
    pub home_screen_settings_color_scheme: Text,
    pub home_screen_settings_auto_updates: Text,
    pub home_screen_settings_language: Text,

    pub home_screen_menu_about: Text,
    pub home_screen_about_credits: Text,
    pub home_screen_credits_content: Text,
    pub home_screen_about_uninstall: Text,
    pub home_screen_uninstall_caption: Text,

    pub home_screen_link_input_placeholder: Text,
   
    pub context_menu_paste: Text,
}

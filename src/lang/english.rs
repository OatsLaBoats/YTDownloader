use super::Translation;

pub fn new() -> Translation {
    Translation {
        general_yes: "Yes",
        general_no: "No",
        general_close: "Close",
            
        update_screen_caption_install: "Installing",
        update_screen_caption_update: "Updating",

        home_screen_update_popup_caption: "Update available",
        home_screen_update_popup_button_update_now: "Update now",
        home_screen_update_pupup_button_update_later: "Update later",

        home_screen_menu_settings: "Settings",
        home_screen_settings_color_scheme: "Color Scheme",
        home_screen_settings_auto_updates: "Auto Updates",
        home_screen_settings_language: "Language",

        home_screen_menu_about: "About",
        home_screen_about_credits: "Credits",
        home_screen_credits_content: "Icons created by:",
        home_screen_about_uninstall: "Uninstall",
        home_screen_uninstall_caption: "Are you sure you want to uninstall?",

        home_screen_link_input_placeholder: "enter link",
       
        context_menu_paste: "paste",

        ..Default::default()
    }
}

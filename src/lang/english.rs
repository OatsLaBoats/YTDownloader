use super::Translation;

pub fn new() -> Translation {
    Translation {
        update_screen_caption_install: "Installing",
        update_screen_caption_update: "Updating",

        home_screen_popup_caption: "Update available",
        home_screen_pupup_button_update_now: "update now",
        home_screen_pupup_button_update_later: "update later",

        home_screen_link_input_placeholder: "enter link",
       
        context_menu_paste: "paste",

        ..Default::default()
    }
}

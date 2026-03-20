use super::Translation;

pub fn new() -> Translation {
    Translation {
        update_screen_install_label: "Installing",
        update_screen_update_label: "Updating",
       
        context_menu_paste: "paste",

        ..Default::default()
    }
}

use super::Translation;

pub fn new() -> Translation {
    Translation {
        general_yes: "Yes",
        general_no: "No",
        general_close: "Close",
            
        update_screen_caption_install: "Installing",
        update_screen_caption_update: "Updating",

        home_screen_update_popup_title: "Update available",
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
       
        context_menu_paste: "Paste link",
        
        general_unknown: "Unknown",
        general_quality: "Quality",
        general_format: "Format",
        general_by: "by",
        general_download: "Download",
        info_panel_link_error: "Error: Invalid URL\nMake sure the link is correct",
        info_panel_media_error: "Error: Failed to retrieve link information\nMake sure that the link refers to valid media",
        info_panel_loading_message_attemp1_label: "Loading link...",
        info_panel_loading_message_attemp2_label: "Retrying...",
        info_panel_playlist_item_placeholder: "Select playlist item...",
        info_panel_download_location_label: "Download location:",
        info_panel_audio_only_checkbox: "Audio only",
        info_panel_side_bar_title: "Downloads",

        tooltip_info_panel_sponsorblock_desc: "Allows you to cut out sponsored segments from videos",
        tooltip_download_close_desc: "Cancel download",
        tooltip_download_open_desc: "Open download folder",

        download_status_downloading: "Downloading",
        download_status_starting: "Starting",
        download_status_postprocessing: "Postprocessing",
        download_status_failed: "Download failed",
        download_status_finished: "Finished",
    }
}

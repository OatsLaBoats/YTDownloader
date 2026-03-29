use super::Translation;

pub fn new() -> Translation {
    Translation {
        general_yes: "ja",
        general_no: "nein",
        general_close: "schließen",
            
        update_screen_caption_install: "installieren",
        update_screen_caption_update: "updating",

        home_screen_update_popup_title: "Update verfügbar",
        home_screen_update_popup_button_update_now: "jetzt updaten",
        home_screen_update_pupup_button_update_later: "später updaten",

        home_screen_menu_settings: "Einstellungen",
        home_screen_settings_color_scheme: "Farbschema",
        home_screen_settings_auto_updates: "automatisches Update",
        home_screen_settings_language: "Sprache",

        home_screen_menu_about: "über",
        home_screen_about_credits: "Credits",
        home_screen_credits_content: "Symbole erstellt von:",
        home_screen_about_uninstall: "löschen",
        home_screen_uninstall_caption: "Soll das Programm wirklich gelöscht werden?",

        home_screen_link_input_placeholder: "Link eingeben",
       
        context_menu_paste: "Link einfügen",
        
        general_unknown: "unbekannt",
        general_quality: "Qualität",
        general_format: "Format",
        general_by: "von",
        general_download: "herunterladen",
        info_panel_link_error: "Fehler: ungültiges URL\nPrüfe, ob der Link korrekt ist",
        info_panel_media_error: "Fehler: Link Information kann nicht geladen werden\nPrüfe, ob der Link sich auf das richtige Media bezieht",
        info_panel_loading_message_attemp1_label: "Link wird geladen...",
        info_panel_loading_message_attemp2_label: "erneuter Versuch...",
        info_panel_playlist_item_placeholder: "wähle playlist item...",
        info_panel_download_location_label: "Speicherplatz:",
        info_panel_audio_only_checkbox: "nur Audio",
        info_panel_side_bar_title: "Downloads",

        tooltip_info_panel_sponsorblock_desc: "Ermöglicht das Ausschneiden von Unerwünschtem",
        tooltip_download_close_desc: "Download abbrechen",
        tooltip_download_open_desc: "Download Ordner öffnen",

        download_status_downloading: "wird geladen",
        download_status_starting: "wird gestarted",
        download_status_re_encoding: "Re-encoding",
        download_status_failed: "Download fehlgeschlagen",
        download_status_finished: "fertig",
    }
}

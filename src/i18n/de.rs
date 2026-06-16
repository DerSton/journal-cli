use super::TrKey;

pub fn translate(key: TrKey) -> &'static str {
    match key {
        // Navigation
        TrKey::NavTitle => " NAVIGATION: ",
        TrKey::NavSwitchHint => "  (Drücke Tab oder 1-3 zum Wechseln)",

        // Tab Names
        TrKey::TabJournal => " ● Journal (1) ",
        TrKey::TabContacts => " ● Kontakte (2) ",
        TrKey::TabSettings => " ● Einstellungen (3) ",

        // List Placeholders
        TrKey::NoEntries => "Keine Einträge in der Datenbank gefunden.",
        TrKey::PressNewEntry => "Drücke 'n', um deinen ersten Eintrag zu schreiben!",
        TrKey::NoContacts => "Keine Kontakte in der Datenbank gefunden.",
        TrKey::PressNewContact => "Drücke 'n', um einen neuen Kontakt hinzuzufügen!",
        TrKey::NoMentions => "Keine Erwähnungen in Journal-Einträgen gefunden.",
        TrKey::MentionHistory => " Erwähnungen ",
        TrKey::JournalEntriesTitle => " Journal-Einträge ",
        TrKey::ContactsListTitle => " Kontakte ",

        // Welcome & Status Messages
        TrKey::WelcomeMsg => "Willkommen in Deinem sicheren Journal-CLI!",
        TrKey::NewEntrySaved => "Neuer Eintrag gespeichert",
        TrKey::EntryUpdated => "Eintrag aktualisiert",
        TrKey::EntryDeleted => "Eintrag gelöscht",
        TrKey::NewContactSaved => "Neuer Kontakt gespeichert",
        TrKey::ContactUpdated => "Kontakt aktualisiert",
        TrKey::ContactDeleted => "Kontakt gelöscht",
        TrKey::PasswordChanged => "Passwort geändert und Datenbank neu verschlüsselt",
        TrKey::SaveFailed => "Speichern fehlgeschlagen",
        TrKey::LocaleUpdated => "Gebietsschema aktualisiert auf",
        TrKey::TimezoneUpdated => "Zeitzone aktualisiert auf",

        // Contact Profile Preview
        TrKey::ProfileTitle => " Kontakt-Profil ",
        TrKey::ProfileFirstName => "  Vorname:     ",
        TrKey::ProfileMiddleName => "  Zweitname:   ",
        TrKey::ProfileLastName => "  Nachname:    ",
        TrKey::ProfileHandle => "  Kürzel:      ",
        TrKey::ProfileBorn => "  Geboren:     ",
        TrKey::ProfileDeceased => "  Gestorben:   ",
        TrKey::ProfileNotes => "  Notizen:",
        TrKey::ProfileAge => "(Alter: {})",
        TrKey::ProfileAged => "(im Alter von {})",

        // Contact Form Editor
        TrKey::FormFirstNameTitle => " Vorname ",
        TrKey::FormMiddleNameTitle => " Zweiter Vorname ",
        TrKey::FormLastNameTitle => " Nachname ",
        TrKey::FormHandleTitle => " Kürzel (für @Erwähnungen) ",
        TrKey::FormBirthdateTitle => " Geburtsdatum ",
        TrKey::FormDeathdateTitle => " Sterbedatum ",
        TrKey::FormNotesTitle => " Notizen ",
        TrKey::FormPressEnterSelect => " [ Eingabetaste drücken zum Auswählen ]",
        TrKey::FormHintNext => "Nächstes Feld",
        TrKey::FormHintPrev => "Vorheriges Feld",
        TrKey::FormHintOpenCalendar => "Kalender öffnen (auf Datumsfeldern)",
        TrKey::FormHintClearDate => "Datum löschen",
        TrKey::FormHintSave => "Kontakt speichern",
        TrKey::FormHintCancel => "Abbrechen",
        TrKey::FormControlsTitle => "Formular-Steuerung:",
        TrKey::FormTitleNew => " ➕  Neuer Kontakt ",
        TrKey::FormTitleEdit => " ✏️  Kontakt bearbeiten ",

        // Settings Options
        TrKey::SettingsHeader => " Einstellungsmenü ",
        TrKey::SettingsPasswordLabel => "🔑  Passwort ändern",
        TrKey::SettingsPasswordDesc => {
            "Ändere das Master-Passwort, welches zum Entschlüsseln der Datenbank genutzt wird."
        }
        TrKey::SettingsLocaleLabel => "🌐  Sprache & Gebietsschema",
        TrKey::SettingsLocaleDesc => {
            "Gebietsschema für die Formatierung von Datum und Uhrzeit festlegen."
        }
        TrKey::SettingsTimezoneLabel => "🕒  Zeitzone",
        TrKey::SettingsTimezoneDesc => "Ziel-Zeitzone konfigurieren.",
        TrKey::SettingsChangePasswordTitle => " Passwort ändern ",
        TrKey::SettingsNewPasswordInput => " Neues Master-Passwort ",
        TrKey::SettingsConfirmPasswordInput => " Neues Passwort bestätigen ",
        TrKey::SettingsSubmitHint => " Änderungen speichern mit Strg + S ",
        TrKey::SettingsSelected => "Aktiv",

        // Help & Buttons Bar
        TrKey::HelpQuit => "Beenden ",
        TrKey::HelpNewEntry => "Neuer Eintrag ",
        TrKey::HelpNewContact => "Neuer Kontakt ",
        TrKey::HelpEdit => "Bearbeiten ",
        TrKey::HelpDelete => "Löschen ",
        TrKey::HelpScrollPreview => "Vorschau scrollen ",
        TrKey::HelpSelectOption => "Option auswählen ",
        TrKey::HelpSelectSave => "Auswählen & Speichern ",
        TrKey::HelpConfirmDelete => "Löschen bestätigen? ",
        TrKey::HelpYesDelete => "Ja, löschen ",
        TrKey::HelpCancel => "Abbrechen ",
        TrKey::HelpNavigate => "Nav ",
        TrKey::HelpMonth => "Monat ",
        TrKey::HelpYear => "Jahr ",
        TrKey::HelpPick => "Wählen ",
        TrKey::HelpClear => "Leeren ",

        // Modals
        TrKey::ModalWarningTitle => " WARNUNG ",
        TrKey::ModalDeleteConfirmQuestion => "Bist du sicher, dass du dies löschen möchtest?",
        TrKey::ModalDeletePermanentWarning => {
            "Diese Aktion ist dauerhaft und kann nicht rückgängig gemacht werden."
        }
        TrKey::ModalDeleteYesBtn => " [y] Ja, löschen ",
        TrKey::ModalDeleteCancelBtn => " [n/Esc] Abbrechen ",
        TrKey::ModalContactPickerTitle => {
            " Kontakt auswählen für Erwähnung [Eingabe: Wählen, Esc: Abbrechen] "
        }
        TrKey::ModalLocaleTitle => " Gebietsschema auswählen ",
        TrKey::ModalTimezoneTitle => " Zeitzone auswählen ",
        TrKey::ModalSearchPrompt => " Suchen: ",

        // Additional Preview
        TrKey::ViewEntryTitle => " Eintrag anzeigen ",
        TrKey::ViewingEntryTitle => "Eintrag anzeigen",
        TrKey::Of => "von",
        TrKey::LabelDate => "Datum: ",
        TrKey::EditorTitleEditEntry => {
            " ✏️  Eintrag bearbeiten [Strg+S: Speichern, Esc: Abbrechen] "
        }
        TrKey::EditorTitleNewEntry => " ➕  Neuer Eintrag [Strg+S: Speichern, Esc: Abbrechen] ",
    }
}

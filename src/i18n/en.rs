use super::TrKey;

pub fn translate(key: TrKey) -> &'static str {
    match key {
        // Navigation
        TrKey::NavTitle => " NAVIGATION: ",
        TrKey::NavSwitchHint => "  (Press Tab or 1-3 to switch)",

        // Tab Names
        TrKey::TabJournal => " ● Journal (1) ",
        TrKey::TabContacts => " ● Contacts (2) ",
        TrKey::TabSettings => " ● Settings (3) ",

        // List Placeholders
        TrKey::NoEntries => "No entries found in database.",
        TrKey::PressNewEntry => "Press 'n' to write your first entry!",
        TrKey::NoContacts => "No contacts found in database.",
        TrKey::PressNewContact => "Press 'n' to add a new contact!",
        TrKey::NoMentions => "No mentions found in journal entries.",
        TrKey::MentionHistory => " Mentions in Journal ",
        TrKey::JournalEntriesTitle => " Journal Entries ",
        TrKey::ContactsListTitle => " Contacts ",

        // Welcome & Status Messages
        TrKey::WelcomeMsg => "Welcome to your secure journal CLI!",
        TrKey::NewEntrySaved => "New entry saved",
        TrKey::EntryUpdated => "Entry updated",
        TrKey::EntryDeleted => "Entry deleted",
        TrKey::NewContactSaved => "New contact saved",
        TrKey::ContactUpdated => "Contact updated",
        TrKey::ContactDeleted => "Contact deleted",
        TrKey::PasswordChanged => "Password changed and database re-encrypted",
        TrKey::SaveFailed => "Save failed",
        TrKey::LocaleUpdated => "Locale updated to",
        TrKey::TimezoneUpdated => "Timezone updated to",

        // Contact Profile Preview
        TrKey::ProfileTitle => " Contact Profile ",
        TrKey::ProfileFirstName => "  First Name:  ",
        TrKey::ProfileMiddleName => "  Middle Name: ",
        TrKey::ProfileLastName => "  Last Name:   ",
        TrKey::ProfileHandle => "  Handle:      ",
        TrKey::ProfileBorn => "  Born:        ",
        TrKey::ProfileDeceased => "  Deceased:    ",
        TrKey::ProfileNotes => "  Notes:",
        TrKey::ProfileAge => "(Age: {})",
        TrKey::ProfileAged => "(Aged: {})",

        // Contact Form Editor
        TrKey::FormFirstNameTitle => " First Name ",
        TrKey::FormMiddleNameTitle => " Middle Name ",
        TrKey::FormLastNameTitle => " Last Name ",
        TrKey::FormHandleTitle => " Handle (for @mentions) ",
        TrKey::FormBirthdateTitle => " Birthdate ",
        TrKey::FormDeathdateTitle => " Date of Death ",
        TrKey::FormNotesTitle => " Notes ",
        TrKey::FormPressEnterSelect => " [ Press Enter to select ]",
        TrKey::FormHintNext => "Next Field",
        TrKey::FormHintPrev => "Prev Field",
        TrKey::FormHintOpenCalendar => "Open Calendar (on Date fields)",
        TrKey::FormHintClearDate => "Clear Date",
        TrKey::FormHintSave => "Save Contact",
        TrKey::FormHintCancel => "Cancel",
        TrKey::FormControlsTitle => "Form Controls:",
        TrKey::FormTitleNew => " ➕  New Contact ",
        TrKey::FormTitleEdit => " ✏️  Edit Contact ",

        // Settings Options
        TrKey::SettingsHeader => " Settings Menu ",
        TrKey::SettingsPasswordLabel => "🔑  Change Password",
        TrKey::SettingsPasswordDesc => {
            "Change master password used to decrypt the journal database."
        }
        TrKey::SettingsLocaleLabel => "🌐  Language & Locale",
        TrKey::SettingsLocaleDesc => "Set application formatting locale for dates and times.",
        TrKey::SettingsTimezoneLabel => "🕒  Timezone",
        TrKey::SettingsTimezoneDesc => "Configure target timezone relative to UTC.",
        TrKey::SettingsChangePasswordTitle => " Change Password ",
        TrKey::SettingsNewPasswordInput => " New Master Password ",
        TrKey::SettingsConfirmPasswordInput => " Confirm New Password ",
        TrKey::SettingsSubmitHint => " Submit changes by pressing Ctrl + S ",
        TrKey::SettingsSelected => "Active",

        // Help & Buttons Bar
        TrKey::HelpQuit => "Quit ",
        TrKey::HelpNewEntry => "New Entry ",
        TrKey::HelpNewContact => "New Contact ",
        TrKey::HelpEdit => "Edit ",
        TrKey::HelpDelete => "Delete ",
        TrKey::HelpScrollPreview => "Scroll Preview ",
        TrKey::HelpSelectOption => "Select Option ",
        TrKey::HelpSelectSave => "Select & Save ",
        TrKey::HelpConfirmDelete => "Confirm Delete? ",
        TrKey::HelpYesDelete => "Yes, Delete ",
        TrKey::HelpCancel => "Cancel ",
        TrKey::HelpNavigate => "Nav ",
        TrKey::HelpMonth => "Month ",
        TrKey::HelpYear => "Year ",
        TrKey::HelpPick => "Pick ",
        TrKey::HelpClear => "Clear ",

        // Modals
        TrKey::ModalWarningTitle => " WARNING ",
        TrKey::ModalDeleteConfirmQuestion => "Are you sure you want to delete this?",
        TrKey::ModalDeletePermanentWarning => "This action is permanent and cannot be undone.",
        TrKey::ModalDeleteYesBtn => " [y] Yes, Delete ",
        TrKey::ModalDeleteCancelBtn => " [n/Esc] Cancel ",
        TrKey::ModalContactPickerTitle => " Select Contact to Mention [Enter: Pick, Esc: Cancel] ",
        TrKey::ModalLocaleTitle => " Select Locale ",
        TrKey::ModalTimezoneTitle => " Select Timezone ",
        TrKey::ModalSearchPrompt => " Search: ",

        // Additional Preview
        TrKey::ViewEntryTitle => " View Entry ",
        TrKey::ViewingEntryTitle => "Viewing Entry",
        TrKey::Of => "of",
        TrKey::LabelDate => "Date: ",
        TrKey::EditorTitleEditEntry => " ✏️  Edit Entry [Ctrl+S: Save, Esc: Cancel] ",
        TrKey::EditorTitleNewEntry => " ➕  New Entry [Ctrl+S: Save, Esc: Cancel] ",
    }
}

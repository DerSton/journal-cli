#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrKey {
    // Navigation
    NavTitle,
    NavSwitchHint,

    // Tab Names
    TabJournal,
    TabContacts,
    TabSettings,

    // List Placeholders
    NoEntries,
    PressNewEntry,
    NoContacts,
    PressNewContact,
    NoMentions,
    MentionHistory,
    JournalEntriesTitle,
    ContactsListTitle,

    // Welcome & Status Messages
    WelcomeMsg,
    NewEntrySaved,
    EntryUpdated,
    EntryDeleted,
    NewContactSaved,
    ContactUpdated,
    ContactDeleted,
    PasswordChanged,
    SaveFailed,
    LocaleUpdated,
    TimezoneUpdated,

    // Contact Profile Preview
    ProfileTitle,
    ProfileFirstName,
    ProfileMiddleName,
    ProfileLastName,
    ProfileHandle,
    ProfileBorn,
    ProfileDeceased,
    ProfileNotes,
    ProfileAge,
    ProfileAged,

    // Contact Form Editor
    FormFirstNameTitle,
    FormMiddleNameTitle,
    FormLastNameTitle,
    FormHandleTitle,
    FormBirthdateTitle,
    FormDeathdateTitle,
    FormNotesTitle,
    FormPressEnterSelect,
    FormHintNext,
    FormHintPrev,
    FormHintOpenCalendar,
    FormHintClearDate,
    FormHintSave,
    FormHintCancel,
    FormControlsTitle,
    FormTitleNew,
    FormTitleEdit,

    // Settings Options
    SettingsHeader,
    SettingsPasswordLabel,
    SettingsPasswordDesc,
    SettingsLocaleLabel,
    SettingsLocaleDesc,
    SettingsTimezoneLabel,
    SettingsTimezoneDesc,
    SettingsChangePasswordTitle,
    SettingsNewPasswordInput,
    SettingsConfirmPasswordInput,
    SettingsSubmitHint,
    SettingsSelected,

    // Help & Buttons Bar
    HelpQuit,
    HelpNewEntry,
    HelpNewContact,
    HelpEdit,
    HelpDelete,
    HelpScrollPreview,
    HelpSelectOption,
    HelpSelectSave,
    HelpConfirmDelete,
    HelpYesDelete,
    HelpCancel,
    HelpNavigate,
    HelpMonth,
    HelpYear,
    HelpPick,
    HelpClear,

    // Modals
    ModalWarningTitle,
    ModalDeleteConfirmQuestion,
    ModalDeletePermanentWarning,
    ModalDeleteYesBtn,
    ModalDeleteCancelBtn,
    ModalContactPickerTitle,
    ModalLocaleTitle,
    ModalTimezoneTitle,
    ModalSearchPrompt,

    // Additional Preview
    ViewEntryTitle,
    ViewingEntryTitle,
    Of,
    LabelDate,
    EditorTitleEditEntry,
    EditorTitleNewEntry,
}

mod de;
mod en;

/// Router function that matches the locale and returns the correct translation string.
pub fn tr(key: TrKey, locale: &str) -> &'static str {
    if locale.starts_with("de") {
        de::translate(key)
    } else {
        en::translate(key)
    }
}

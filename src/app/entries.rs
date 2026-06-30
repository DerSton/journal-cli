//! App state controller implementations for journal entry actions (saving and deleting).

use super::{App, AppMode};
use crate::model::JournalEntry;
use chrono::Utc;
use uuid::Uuid;

impl App {
    /// Saves the entry currently in the text area (creating or updating), persisting it to disk.
    pub fn save_entry(&mut self) {
        let content = self.textarea.lines().join("\n");
        if content.trim().is_empty() {
            self.error_msg = Some("Error: Entry content cannot be empty".to_string());
            return;
        }

        match self.mode {
            AppMode::Writing { is_edit: false } => {
                self.journal.entries.push(JournalEntry {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    content,
                    date_for: self.entry_date_for,
                    attachments: Vec::new(),
                });
                self.sort_entries();
                self.selected_index = 0;
                self.status_msg = Some("New entry saved".to_string());
            }
            AppMode::Writing { is_edit: true } => {
                let mut edited_id = None;
                if let Some(entry) = self
                    .selected_entry_idx()
                    .and_then(|idx| self.journal.entries.get_mut(idx))
                {
                    entry.content = content;
                    entry.date_for = self.entry_date_for;
                    edited_id = Some(entry.id.clone());
                    self.status_msg = Some("Entry updated".to_string());
                }
                self.sort_entries();
                if let Some(pos) =
                    edited_id.and_then(|id| self.filtered_entries().iter().position(|e| e.id == id))
                {
                    self.selected_index = pos;
                }
            }
            _ => return,
        }

        if let Err(e) = self.save_journal() {
            self.error_msg = Some(format!("Write failed: {}", e));
        } else {
            self.mode = AppMode::List;
            self.detail_scroll = 0;
            self.error_msg = None;
        }
    }

    /// Deletes the currently selected journal entry, updates selection indices, and persists changes to disk.
    pub fn delete_selected_entry(&mut self) {
        let real_idx = match self.selected_entry_idx() {
            Some(idx) => idx,
            None => {
                self.mode = AppMode::List;
                return;
            }
        };

        self.journal.entries.remove(real_idx);

        if let Err(e) = self.save_journal() {
            self.error_msg = Some(format!("Delete write failed: {}", e));
        } else {
            self.status_msg = Some("Entry deleted".to_string());
            self.error_msg = None;
        }

        let len = self.filtered_entries().len();
        if len == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= len {
            self.selected_index = len - 1;
        }

        self.mode = AppMode::List;
        self.detail_scroll = 0;
    }

    /// Size threshold in bytes above which a warning is shown for attached files.
    const LARGE_FILE_THRESHOLD: u64 = 5 * 1024 * 1024; // 5 MiB

    /// Opens a native file picker, reads selected files, and attaches them to
    /// the currently selected journal entry as Base64-encoded blobs.
    ///
    /// Temporarily suspends the TUI to allow the OS file dialog to render.
    /// Files larger than [`Self::LARGE_FILE_THRESHOLD`] still attach but
    /// trigger a status-bar warning.
    pub fn attach_files(&mut self) {
        let real_idx = match self.selected_entry_idx() {
            Some(idx) => idx,
            None => {
                self.error_msg = Some("No entry selected".to_string());
                return;
            }
        };

        // Suspend TUI so the native dialog can render.
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);

        let files = std::thread::spawn(|| {
            rfd::FileDialog::new()
                .set_title("Attach files to entry")
                .pick_files()
        })
        .join()
        .unwrap_or(None);

        // Restore TUI.
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen);
        let _ = crossterm::terminal::enable_raw_mode();
        self.redraw_requested = true;

        let paths = match files {
            Some(p) if !p.is_empty() => p,
            _ => {
                self.status_msg = Some("No files selected".to_string());
                return;
            }
        };

        let mut attached = 0usize;
        let mut large_files: Vec<String> = Vec::new();

        for path in &paths {
            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(e) => {
                    self.error_msg = Some(format!("Failed to read {}: {}", path.display(), e));
                    return;
                }
            };

            let size = data.len() as u64;
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown".to_string());

            if size > Self::LARGE_FILE_THRESHOLD {
                large_files.push(format!(
                    "{} ({:.1} MiB)",
                    filename,
                    size as f64 / (1024.0 * 1024.0)
                ));
            }

            let mime_type = mime_from_extension(path);
            let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);

            self.journal.entries[real_idx]
                .attachments
                .push(crate::model::Attachment {
                    filename,
                    mime_type,
                    size_bytes: size,
                    data: encoded,
                });

            attached += 1;
        }

        if let Err(e) = self.save_journal() {
            self.error_msg = Some(format!("Write failed: {}", e));
            return;
        }

        if !large_files.is_empty() {
            self.status_msg = Some(format!(
                "{} file(s) attached — large: {}",
                attached,
                large_files.join(", ")
            ));
        } else {
            self.status_msg = Some(format!("{} file(s) attached", attached));
        }
        self.error_msg = None;
    }

    /// Exports the currently selected journal entry as a Markdown file via
    /// a native save-file dialog.
    ///
    /// The export contains only text content (no attachments).
    pub fn export_entry_as_md(&mut self) {
        let filtered = self.filtered_entries();
        let entry = match filtered.get(self.selected_index) {
            Some(e) => e,
            None => {
                self.error_msg = Some("No entry selected".to_string());
                return;
            }
        };

        let date_str = if entry.date_for.is_some() {
            self.journal.format_date(&entry.sort_timestamp())
        } else {
            self.journal.format_timestamp(&entry.timestamp)
        };

        let md = format!("# Journal Entry\n\n**{}**\n\n{}\n", date_str, entry.content);

        let default_name = format!(
            "journal_{}.md",
            if let Some(d) = entry.date_for {
                d.format("%Y-%m-%d").to_string()
            } else {
                entry.timestamp.format("%Y-%m-%d_%H%M%S").to_string()
            }
        );

        // Suspend TUI for native dialog.
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);

        let dest = std::thread::spawn(move || {
            rfd::FileDialog::new()
                .set_title("Export entry as Markdown")
                .set_file_name(&default_name)
                .add_filter("Markdown", &["md"])
                .save_file()
        })
        .join()
        .unwrap_or(None);

        // Restore TUI.
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen);
        let _ = crossterm::terminal::enable_raw_mode();
        self.redraw_requested = true;

        let path = match dest {
            Some(p) => p,
            None => {
                self.status_msg = Some("Export cancelled".to_string());
                return;
            }
        };

        match std::fs::write(&path, md.as_bytes()) {
            Ok(_) => {
                self.status_msg = Some(format!("Exported to {}", path.display()));
                self.error_msg = None;
            }
            Err(e) => {
                self.error_msg = Some(format!("Export failed: {}", e));
            }
        }
    }
}

/// Infers a MIME type from a file's extension.
fn mime_from_extension(path: &std::path::Path) -> String {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("jpg" | "jpeg") => "image/jpeg".to_string(),
        Some("png") => "image/png".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("webp") => "image/webp".to_string(),
        Some("svg") => "image/svg+xml".to_string(),
        Some("pdf") => "application/pdf".to_string(),
        Some("txt") => "text/plain".to_string(),
        Some("md") => "text/markdown".to_string(),
        Some("json") => "application/json".to_string(),
        Some("csv") => "text/csv".to_string(),
        Some("zip") => "application/zip".to_string(),
        Some("mp3") => "audio/mpeg".to_string(),
        Some("mp4") => "video/mp4".to_string(),
        Some("wav") => "audio/wav".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

#[cfg(test)]
mod entries_tests {
    use super::*;
    use crate::model::Attachment;
    use std::path::Path;

    #[test]
    fn test_mime_inference() {
        assert_eq!(mime_from_extension(Path::new("photo.jpg")), "image/jpeg");
        assert_eq!(
            mime_from_extension(Path::new("document.pdf")),
            "application/pdf"
        );
        assert_eq!(
            mime_from_extension(Path::new("archive.zip")),
            "application/zip"
        );
        assert_eq!(
            mime_from_extension(Path::new("unknown.xyz")),
            "application/octet-stream"
        );
        assert_eq!(mime_from_extension(Path::new("notes.md")), "text/markdown");
    }

    #[test]
    fn test_attachment_struct_serialization() {
        let attachment = Attachment {
            filename: "test.txt".to_string(),
            mime_type: "text/plain".to_string(),
            size_bytes: 12,
            data: "SGVsbG8gV29ybGQ=".to_string(), // "Hello World" in base64
        };

        let serialized = serde_json::to_string(&attachment).unwrap();
        let deserialized: Attachment = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.filename, "test.txt");
        assert_eq!(deserialized.mime_type, "text/plain");
        assert_eq!(deserialized.size_bytes, 12);
        assert_eq!(deserialized.data, "SGVsbG8gV29ybGQ=");
    }
}

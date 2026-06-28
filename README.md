# journal-cli

A private, encrypted journal you keep in a terminal. Single binary, single file, no server, no cloud.

```
jnl my-journal.jrnl
```

## Features

- **Encrypted at rest** — ChaCha20Poly1305 with an Argon2id-derived key (OWASP-recommended params). Your entries never touch disk in plaintext.
- **Terminal UI** — fast, keyboard-driven, built with [ratatui](https://github.com/ratatui-org/ratatui).
- **Dashboard tab** — landing dashboard displaying summary statistics (Total entries, total words, last entry date, current/record streaks) and a weekly AI-generated chronological summary of your recent days (powered by local Ollama).
- **Journal tab** — write and browse entries, newest first. Run local **KI-Analyse** (AI analysis) on any selected entry by pressing `a`.
- **Contacts tab** — rich contact records (names, pronouns, languages, birthdate, notes, ...) sorted alphabetically. Birthdate and Date of Death inputs are fully localized and automatically adapt to your system's date format (e.g. `DD.MM.YYYY`). Mention a contact in an entry with `{{person|<uuid>}}` and it renders as a highlighted name.
- **Stats tab** — view entry streaks, total and average word counts, top contact mentions, and a word-count history chart for recent entries.
- **Settings tab** — change your master password, set an inactivity auto-lock timeout, lock on workstation suspend (Windows), generate recovery shares, toggle Ollama weekly summaries, and cycle through available local Ollama models.
- **Local AI Analysis (KI-Analyse)** — fully offline analysis of journal entries. Press `a` in the Journal list to perform:
  - _Spelling & Formatting corrections_ with a visual diff confirmation pane.
  - _Automatic contact linking_ (linking names in text to database contacts with manual confirmation).
  - _Strict tag suggestions_ stored separately in the database JSON.
- **Password recovery** — master password can be split into Shamir secret-sharing shares, so you can recover access without ever storing the password itself.
- **Transactional saves** — password changes and journal writes go through a temp-file + rename, so a crash mid-write can't corrupt your journal.

## Installation

The installer scripts download the `journal-cli` binary and automatically configure a convenient `jnl` alias for you. You can run the journal using either `journal-cli` or `jnl`.

### Shell Script (Linux)

Install or update to the latest version by running:

```bash
curl -fsSL https://raw.githubusercontent.com/DerSton/journal-cli/main/install.sh | bash
```

### PowerShell (Windows)

Open PowerShell and run:

```powershell
irm https://raw.githubusercontent.com/DerSton/journal-cli/main/install.ps1 | iex
```

### Manual Download

Grab a prebuilt binary from the [Releases](https://github.com/DerSton/journal-cli/releases) page (Linux x86_64, Windows x86_64).

### Build from source

```
git clone https://github.com/DerSton/journal-cli.git
cd journal-cli
cargo build --release
```

The binary will be at `target/release/journal-cli` (or `journal-cli.exe` on Windows).

## Usage

```
jnl [JOURNAL_PATH]
```

`JOURNAL_PATH` defaults to `journal.jrnl` in the current directory if omitted.

- If the file doesn't exist yet, you'll be prompted to set a master password and a new encrypted journal is created there.
- If it exists, you'll be prompted to log in with your master password.

Run `jnl --help` for the full option list, `jnl --version` for the version.

### Key bindings

The app is keyboard-driven; on-screen hints show available actions per mode.

- **Tab switching**: Press `Tab` or numbers `1`–`5` to switch directly between tabs (1: Dashboard, 2: Journal, 3: Contacts, 4: Stats, 5: Settings).
- **Dashboard**: Press `r` to regenerate the weekly Ollama summary. Use `Up`/`Down`/`PageUp`/`PageDown`/`j`/`k` to scroll the summary text.
- **Journal**: Press `n` for new entry, `e` to edit, `d`/`Delete` to delete, and `a` to run local KI-Analyse (AI analysis).
- **Journal editor**: Press `Alt+p` to insert a contact mention, `Alt+d` to select an optional back-date (gilt für), `Ctrl+s` to save, and `Esc` to cancel.
- **KI-Analyse Review**: Use `Tab` to switch focus between Tags, Contacts, and Diff panels. Press `Space` to toggle selections, `Ctrl+S` to apply, and `Esc` to cancel.

## File format

A journal file starts with the magic bytes `JRNL`, followed by a 16-byte salt, a 12-byte nonce, and a ChaCha20Poly1305-encrypted payload (JSON internally). There is no way to read the contents without the master password.

## Security notes

- The master password is held in memory only for the duration of the session (needed to re-encrypt on save) and is never written to disk.
- A fresh nonce is generated on every save.
- Recovery shares use Shamir secret sharing — losing the master password is recoverable only if you generated and stored shares in advance.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

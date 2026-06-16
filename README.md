# journal-cli

A private, encrypted journal you keep in a terminal. Single binary, single file, no server, no cloud.

```
journal-cli my-journal.jrnl
```

## Features

- **Encrypted at rest** — ChaCha20Poly1305 with an Argon2id-derived key (OWASP-recommended params). Your entries never touch disk in plaintext.
- **Terminal UI** — fast, keyboard-driven, built with [ratatui](https://github.com/ratatui-org/ratatui).
- **Journal tab** — write and browse entries, newest first.
- **Contacts tab** — rich contact records (names, pronouns, languages, birthdate, notes, ...) sorted alphabetically. Mention a contact in an entry with `{{person|<uuid>}}` and it renders as a highlighted name.
- **Settings tab** — change your master password, set an inactivity auto-lock timeout, lock on workstation suspend (Windows), generate recovery shares.
- **Password recovery** — master password can be split into Shamir secret-sharing shares, so you can recover access without ever storing the password itself.
- **Transactional saves** — password changes and journal writes go through a temp-file + rename, so a crash mid-write can't corrupt your journal.

## Installation

### Download a release

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
journal-cli <path-to-journal-file>
```

- If the file doesn't exist yet, you'll be prompted to set a master password and a new encrypted journal is created.
- If it exists, you'll be prompted to log in with your master password.

### Key bindings

The app is keyboard-driven; on-screen hints show available actions per mode (writing, browsing, contact editing, settings). Tab/Shift+Tab switches between Journal, Contacts, and Settings tabs.

## File format

A journal file starts with the magic bytes `JRNL`, followed by a 16-byte salt, a 12-byte nonce, and a ChaCha20Poly1305-encrypted payload (JSON internally). There is no way to read the contents without the master password.

## Security notes

- The master password is held in memory only for the duration of the session (needed to re-encrypt on save) and is never written to disk.
- A fresh nonce is generated on every save.
- Recovery shares use Shamir secret sharing — losing the master password is recoverable only if you generated and stored shares in advance.

## License

No license specified yet.

# journal-cli

A private, encrypted journal in your terminal. Single binary, single file, no server, no cloud.

## Features

- **Encryption**: ChaCha20Poly1305 with an Argon2id-derived key (OWASP params).
- **TUI**: Keyboard-driven interface built with Ratatui.
- **Tabs**:
  - **Journal**: Write/browse entries and link contacts with `{{person|<uuid>}}`.
  - **People**: Keep localized contact records (birthdate, notes, etc.).
  - **Insights**: Streak counters, word counts, and frequent words chart.
  - **Settings**: Password changes, inactivity timeouts, lock on suspend (Windows), and Shamir recovery shares.
- **Safety**: Shamir Secret Sharing recovery shares and transactional saves.

## Installation

### Linux & macOS

```bash
curl -fsSL https://raw.githubusercontent.com/DerSton/journal-cli/main/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/DerSton/journal-cli/main/install.ps1 | iex
```

### Build from source

```bash
git clone https://github.com/DerSton/journal-cli.git
cd journal-cli
cargo build --release
```

The binary will be at `target/release/journal-cli` (or `journal-cli.exe` on Windows).

## Usage

```bash
jnl [JOURNAL_PATH]
```

_(Defaults to `journal.jrnl` in the current directory if omitted. Prompts to set master password on first launch)._

## License

MIT License - see [LICENSE](LICENSE) for details.

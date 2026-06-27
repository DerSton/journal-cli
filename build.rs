fn main() {
    // If a version is provided via environment variable (e.g. from GitHub Actions release tag),
    // we use it. Otherwise we fall back to the version specified in Cargo.toml.
    let version = std::env::var("JOURNAL_CLI_VERSION")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());

    println!("cargo:rustc-env=APP_VERSION={}", version);
}

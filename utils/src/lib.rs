use owo_colors::OwoColorize;

pub fn version(bin_name: &str) -> String {
    format!(
        "{}, {} v{} ({}, {})",
        bin_name,
        env!("CARGO_PKG_NAME").cyan(),
        env!("CARGO_PKG_VERSION").blue(),
        (&env!("VERGEN_GIT_SHA")[0..=6]).green(),
        env!("VERGEN_GIT_COMMIT_DATE").green(),
    )
}

pub fn to_pretty_size(size: u64) -> String {
    if size < 1024 {
        size.to_string() + " B"
    } else if size < 1024u64.pow(2) {
        (size / 1024).to_string() + " kiB"
    } else if size < 1024u64.pow(3) {
        (size / 1024u64.pow(2)).to_string() + " MiB"
    } else if size < 1024u64.pow(4) {
        (size / 1024u64.pow(3)).to_string() + " GiB"
    } else if size < 1024u64.pow(5) {
        (size as u128 / 1024u128.pow(4)).to_string() + " TiB"
    } else if size < 1024u64.pow(6) {
        (size as u128 / 1024u128.pow(5)).to_string() + " PiB"
    } else if size < 1024u64.pow(7) {
        (size as u128 / 1024u128.pow(6)).to_string() + " EiB"
    } else {
        size.to_string() + " B"
    }
}

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

//! {{project-name}} executable.

#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::let_underscore_untyped,
    clippy::map_unwrap_or,
    clippy::module_name_repetitions
)]
// Require docs on everything
#![warn(missing_docs, clippy::missing_docs_in_private_items)]
// Other restriction lints
#![warn(clippy::arithmetic_side_effects)]

use anyhow::bail;
use clap::Parser;
use simplelog::{
    ColorChoice, CombinedLogger, Config, ConfigBuilder, LevelFilter,
    TermLogger, TerminalMode,
};
use std::process::exit;

/// Parameters to configure executable.
#[derive(Debug, clap::Parser)]
#[clap(version, about)]
struct Params {
    /// Verbosity (may be repeated up to three times)
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    smol::block_on(async {
        if let Err(error) = cli(Params::parse()).await {
            eprintln!("Error: {error:#}");
            exit(1);
        }
    });
}

/// Set up and run executable.
#[allow(clippy::unused_async)]
async fn cli(params: Params) -> anyhow::Result<()> {
    let filter = match params.verbose {
        4.. => bail!("-v is only allowed up to 3 times."),
        3 => LevelFilter::Trace,
        2 => LevelFilter::Debug,
        1 => LevelFilter::Info,
        0 => LevelFilter::Warn,
    };

    // Configure different logging for a module (sqlx::query here).
    CombinedLogger::init(vec![
        // Default logger
        new_term_logger(
            filter,
            new_logger_config()
                .add_filter_ignore_str("sqlx::query")
                .build(),
        ),
        // Logger for sqlx::query
        new_term_logger(
            LevelFilter::Warn,
            new_logger_config()
                .add_filter_allow_str("sqlx::query")
                .build(),
        ),
    ])
    .unwrap();

    Ok(())
}

/// Convenience function to make creating [`TermLogger`]s clearer.
#[allow(clippy::unnecessary_box_returns)] // Using `Box` isn’t our decision.
fn new_term_logger(level: LevelFilter, config: Config) -> Box<TermLogger> {
    TermLogger::new(level, config, TerminalMode::Mixed, ColorChoice::Auto)
}

/// Convenience function to make creating [`ConfigBuilder`]s clearer.
fn new_logger_config() -> ConfigBuilder {
    let mut builder = ConfigBuilder::new();
    builder.set_target_level(LevelFilter::Error);

    // FIXME: If this fails it will just print the time in UTC. That might be a
    // little surprising, so this should probably warn the user... except that
    // we haven’t finished setting up logging.
    let _ = builder.set_time_offset_to_local();

    builder
}

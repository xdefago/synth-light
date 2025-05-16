use anyhow::Result;
use clap::Parser;
use synth_lights::Cli;

use log::info;

use simplelog::*;

const RUST_LOG: &str = "RUST_LOG";

fn env_loglevel() -> LevelFilter {
    let log_level = std::env::var_os(RUST_LOG).map(|s| s.to_string_lossy().to_lowercase());
    match log_level.as_deref() {
        Some("off") | None => LevelFilter::Off,
        Some("trace") => LevelFilter::Trace,
        Some("debug") => LevelFilter::Debug,
        Some("info") => LevelFilter::Info,
        Some("warn") => LevelFilter::Warn,
        Some("error") => LevelFilter::Error,
        Some(s) => panic!("Unrecognized error level in RUST_LOG: {}", s),
    }
}

fn main() -> Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(
            env_loglevel(),
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            env_loglevel(),
            Config::default(),
            std::fs::File::create("synth-lights.log").unwrap(),
        ),
    ])
    .unwrap();

    let cli = Cli::parse();

    info!("Run options: {:?}", cli);

    synth_lights::run(&cli)
}

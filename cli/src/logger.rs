use std::{io, io::Write};
use tracing::subscriber;
use tracing_subscriber::{fmt, fmt::time::ChronoLocal, prelude::*};

pub fn setup_logger(verbose: bool) {
    let env = tracing_subscriber::EnvFilter::from_env("ROAN_LOG");

    // Set common time format
    let time_format = if verbose {
        "%Y-%m-%d %H:%M:%S%.3f"
    } else {
        "%H:%M:%S%.3f"
    };

    let console_layer = fmt::Layer::new()
        .with_writer(io::stderr)
        .with_timer(ChronoLocal::new(time_format.into()))
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
        .with_target(verbose);

    if verbose {
        let file_appender = tracing_appender::rolling::hourly(
            concat!(env!("CARGO_MANIFEST_DIR"), "/logs"),
            "roan-cli.log",
        );

        let subscriber = tracing_subscriber::Registry::default()
            .with(console_layer)
            .with(
                fmt::Layer::new()
                    .with_writer(file_appender)
                    .with_timer(ChronoLocal::new(time_format.into()))
                    .with_ansi(false)
                    .with_target(true)
                    .with_line_number(true)
                    .with_file(true),
            )
            .with(env);

        subscriber::set_global_default(subscriber).expect("Failed to set logger");
    } else {
        let subscriber = tracing_subscriber::Registry::default()
            .with(console_layer)
            .with(env);
        subscriber::set_global_default(subscriber).expect("Failed to set logger");
    };
}

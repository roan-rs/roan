pub fn setup_logger() {
    use tracing_subscriber::prelude::*;

    let env = tracing_subscriber::EnvFilter::from_env("ROAN_LOG");
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_timer(tracing_subscriber::fmt::time::Uptime::default())
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
        .with_writer(std::io::stderr)
        .with_filter(env);

    let subscriber = tracing_subscriber::Registry::default().with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set logger");
}

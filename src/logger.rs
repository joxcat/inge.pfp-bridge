use tracing_subscriber::{fmt, prelude::*, util::SubscriberInitExt, EnvFilter};

pub fn setup_logger() {
    tracing_subscriber::Registry::default()
        .with(fmt::layer().pretty().with_ansi(true))
        .with(EnvFilter::from_env("LOG_LEVEL"))
        .init();
}

pub fn slice_to_hex(slice: &[u8]) -> String {
    slice
        .iter()
        .map(|&b| format!("0x{b:02x}"))
        .collect::<Vec<String>>()
        .join(",")
}

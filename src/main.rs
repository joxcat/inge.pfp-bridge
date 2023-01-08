use std::env::set_var;

use clap::Parser;
use logger::setup_logger;
use mqtt::SimpleMQTT;
use serial::SimpleSerial;
use tracing::{debug, info, warn};

use crate::simulation::Simulation;

mod logger;
mod mqtt;
mod protocol;
mod protocol_parser;
mod serial;
mod simulation;

pub type Result<T> = eyre::Result<T>;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Serial port path
    #[clap(short = 'p', long, env)]
    pub serial_port: String,
    /// Serial baud rate, default to the microbit default baud rate
    #[clap(short = 'b', long, env, default_value = "115200")]
    pub serial_baud_rate: u32,
    #[clap(short, long = "verbose", action = clap::ArgAction::Count)]
    pub verbosity: u8,
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Parser)]
enum SubCommand {
    /// Bridge to MQTT
    Mqtt(CliMqttBridge),
    /// Bridge to network
    Network(CliNetworkBridge),
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct CliMqttBridge {
    /// MQTT server host
    #[clap(short = 'H', long, env, default_value = "localhost")]
    pub mqtt_host: String,
    /// MQTT server port
    #[clap(short = 'P', long, env, default_value = "1883")]
    pub mqtt_port: u16,
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct CliNetworkBridge {
    /// Simulation server
    #[clap(short = 's', long, env, default_value = "localhost:8080")]
    pub simulation_server: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().unwrap();

    let args = Cli::parse();
    if args.verbosity > 2 {
        set_var("LOG_LEVEL", "trace");
    } else if args.verbosity == 2 {
        set_var("LOG_LEVEL", "debug");
    } else if args.verbosity == 1 {
        set_var("LOG_LEVEL", "info");
    }
    setup_logger();

    let mut serial = SimpleSerial::new(&args.serial_port, args.serial_baud_rate)?;
    info!(
        serial = args.serial_port,
        baudrate = args.serial_baud_rate,
        "Connected to serial port"
    );

    match args.subcommand {
        SubCommand::Mqtt(args) => {
            let mut mqtt = SimpleMQTT::new(&args.mqtt_host, args.mqtt_port).await?;
            info!(
                mqtt = args.mqtt_host,
                port = args.mqtt_port,
                "Connected to MQTT server"
            );
        }
        SubCommand::Network(args) => {
            let mut simulation = Simulation::new(&args.simulation_server)?;
        }
    }

    while let Ok(line) = serial.read_line() {
        if let Ok(pfp_req) = protocol_parser::parse(&line) {
        } else {
            warn!("Failed to parse protocol");
            debug!("Line: {:x?}", line);
        }
        // TODO: Send to mqtt
        // mqtt.push("microbit", &line).await?;
    }

    Ok(())
}

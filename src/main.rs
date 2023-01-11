use std::env::set_var;

use clap::Parser;
use logger::setup_logger;
use mqtt::SimpleMQTT;
use serial::SimpleSerial;
use tracing::{debug, info, warn};

use crate::{protocol::Command, simulation::Simulation};

mod logger;
mod mqtt;
mod protocol;
mod protocol_parser;
mod read_until;
mod serial;
mod simulation;

pub type Result<T> = eyre::Result<T>;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Serial port path
    #[clap(short = 'p', long, env)]
    pub serial_port: String,
    /// Serial baud rate
    #[clap(short = 'b', long, env, default_value = "38400")]
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
    /// Read UART
    Debug,
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
    /// MQTT channel
    #[clap(short = 'C', long, env, default_value = "microbit")]
    pub mqtt_channel: String,
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

            while let Ok(line) = serial.read_line() {
                if let Ok((_, request)) = protocol_parser::parse(&line) {
                    if request.command_id == Command::Push {
                        if let Ok((_, (device_serial, intensity))) =
                            protocol_parser::parse_push_payload(&request.payload)
                        {
                            mqtt.push(
                                &args.mqtt_channel,
                                &format!("telegraf id={device_serial},intensity={intensity}"),
                            )
                            .await?;
                        }
                    }
                } else {
                    warn!(
                        line = String::from_utf8(line).unwrap_or_else(|_| String::new()),
                        "Invalid request"
                    );
                }
            }
        }
        SubCommand::Network(args) => {
            let mut simulation = Simulation::new(&args.simulation_server)?;
        }
        SubCommand::Debug => {
            while let Ok(line) = serial.read_line() {
                debug!(
                    "{}",
                    String::from_utf8(line).unwrap_or_else(|_| String::new())
                );
            }
        }
    }

    Ok(())
}

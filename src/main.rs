use std::{env::set_var, sync::Arc};

use clap::Parser;
use logger::setup_logger;
use mosquitto_rs::Message;
use mqtt::SimpleMQTT;
use serial::SimpleSerial;
use tokio::{sync::oneshot, sync::RwLock};
use tracing::{debug, info, warn};

use crate::{
    protocol::{Command, PFPRequest},
    relay::Relay,
};

mod logger;
mod mqtt;
mod protocol;
mod protocol_parser;
mod read_until;
mod relay;
mod serial;

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
    /// Bridge as a relay
    Relay(CliRelay),
    /// Bridge to simulation
    Simulator(CliSimulation),
    /// Read UART
    Debug,
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct CliRelay {
    /// MQTT server host
    #[clap(short = 'H', long, env, default_value = "localhost")]
    pub mqtt_host: String,
    /// MQTT server port
    #[clap(short = 'P', long, env, default_value = "1883")]
    pub mqtt_port: u16,
    /// MQTT channel
    #[clap(short = 'C', long, env, default_value = "microbit/manager")]
    pub mqtt_channel: String,
    /// Do not connect to MQTT server
    #[clap(long, env, action = clap::ArgAction::SetTrue)]
    pub dry_mqtt: bool,
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct CliSimulation {
    /// MQTT server host
    #[clap(short = 'H', long, env, default_value = "localhost")]
    pub mqtt_host: String,
    /// MQTT server port
    #[clap(short = 'P', long, env, default_value = "1883")]
    pub mqtt_port: u16,
    /// MQTT channel
    #[clap(short = 'C', long, env, default_value = "microbit/simulator")]
    pub mqtt_channel: String,
    /// Do not connect to MQTT server
    #[clap(long, env, action = clap::ArgAction::SetTrue)]
    pub dry_mqtt: bool,
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

    let (shutdown_trigger, mut shutdown_signal) = oneshot::channel::<()>();
    {
        tokio::spawn(async move {
            debug!("Spawned Ctrl-C handler task");
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C signal handler");

            warn!("Received Ctrl-C, shutting down");
            shutdown_trigger
                .send(())
                .expect("Failed to send stop signal");
        });
    }

    let mut serial = SimpleSerial::new(&args.serial_port, args.serial_baud_rate)?;
    info!(
        serial = args.serial_port,
        baudrate = args.serial_baud_rate,
        "Connected to serial port"
    );

    match args.subcommand {
        SubCommand::Relay(args) => {
            let mqtt = Arc::new(RwLock::new(
                SimpleMQTT::new(&args.mqtt_host, args.mqtt_port, args.dry_mqtt).await?,
            ));
            info!(
                mqtt = args.mqtt_host,
                port = args.mqtt_port,
                "Connected to MQTT server"
            );
            let serial = Arc::new(RwLock::new(serial));
            let mqtt_channel = Arc::new(args.mqtt_channel);

            Relay::new(
                1,
                serial,
                move |request: Arc<PFPRequest>| {
                    let mqtt_channel = mqtt_channel.clone();
                    let mqtt = mqtt.clone();
                    Box::pin(async move {
                        if request.command_id == Command::Push {
                            if let Ok((_, (device_serial, intensity))) =
                                protocol_parser::parse_push_payload(&request.payload)
                            {
                                mqtt.write()
                                    .await
                                    .push(
                                        &mqtt_channel,
                                        &format!(
                                            "telegraf serial={device_serial},intensity={intensity}"
                                        ),
                                    )
                                    .await?;
                            }
                        }
                        Ok(())
                    })
                },
                shutdown_signal,
            )
            .run()
            .await?;
        }
        SubCommand::Simulator(args) => {
            let mqtt = Arc::new(RwLock::new(
                SimpleMQTT::new(&args.mqtt_host, args.mqtt_port, args.dry_mqtt).await?,
            ));
            info!(
                mqtt = args.mqtt_host,
                port = args.mqtt_port,
                "Connected to MQTT server"
            );
            mqtt.write().await.subscribe(&args.mqtt_channel).await?;

            {
                let mqtt_outer = mqtt.clone();
                let mqtt_inner = mqtt.clone();
                { mqtt_outer.write().await }
                    .on_message(
                        Box::new(move |message: Message| {
                            let mqtt = mqtt_inner.clone();
                            Box::pin(async move {
                                // TODO: Do something with MQTT
                                mqtt.write()
                                    .await
                                    .push(
                                        "microbit/manager",
                                        &String::from_utf8(message.payload)
                                            .unwrap_or_else(|_| String::new()),
                                    )
                                    .await?;
                                Ok::<_, eyre::Report>(())
                            })
                        }),
                        shutdown_signal,
                    )
                    .await?;
            }
        }
        SubCommand::Debug => {
            while shutdown_signal.try_recv().is_err() {
                serial.write_buf(b"test").unwrap();
                if let Ok(line) = serial.read_line() {
                    debug!(
                        "{}",
                        String::from_utf8(line).unwrap_or_else(|_| String::new())
                    );
                }
            }
        }
    }

    Ok(())
}

use std::time::Duration;

use mosquitto_rs::Message;
use tokio::sync::oneshot;
use tracing::{debug, info};

pub struct SimpleMQTT {
    client: mosquitto_rs::Client,
    dry_run: bool,
}

impl SimpleMQTT {
    pub async fn new(host: &str, port: u16, dry_run: bool) -> crate::Result<Self> {
        let mut client = mosquitto_rs::Client::with_auto_id()?;
        if !dry_run {
            client
                .connect(host, port as i32, Duration::from_secs(5), None)
                .await?;
        }

        Ok(Self { client, dry_run })
    }

    pub async fn push(&mut self, topic: &str, payload: &str) -> crate::Result<()> {
        info!(topic, payload, "Pushing to MQTT");

        if !self.dry_run {
            self.client
                .publish(
                    topic,
                    payload.as_bytes(),
                    mosquitto_rs::QoS::AtLeastOnce,
                    false,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn subscribe(&mut self, topic: &str) -> crate::Result<()> {
        info!(topic, "Subscribing to MQTT");

        if !self.dry_run {
            self.client
                .subscribe(topic, mosquitto_rs::QoS::AtLeastOnce)
                .await?;
        }

        Ok(())
    }

    pub async fn on_message(
        &mut self,
        on_message: impl Fn(Message) -> crate::Result<()>,
        shutdown_signal: oneshot::Receiver<()>,
    ) -> crate::Result<()> {
        if !self.dry_run {
            if let Some(subscriber) = self.client.subscriber() {
                let shutdown_signal = shutdown_signal;
                tokio::pin!(shutdown_signal);

                loop {
                    tokio::select! {
                        message = &mut subscriber.recv() => {
                            if let Ok(message) = message {
                                debug!(topic = &message.topic, payload = String::from_utf8(message.payload.clone()).unwrap_or_else(|_| String::new()), "Received MQTT Message");
                                on_message(message)?
                            }
                        }
                        _ = &mut shutdown_signal => {
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

use std::time::Duration;

pub struct SimpleMQTT {
    client: mosquitto_rs::Client,
}

impl SimpleMQTT {
    pub async fn new(host: &str, port: u16) -> crate::Result<Self> {
        let mut client = mosquitto_rs::Client::with_auto_id()?;
        client
            .connect(host, port as i32, Duration::from_secs(5), None)
            .await?;

        Ok(Self { client })
    }

    pub async fn push(&mut self, topic: &str, payload: &str) -> crate::Result<()> {
        self.client
            .publish(
                topic,
                payload.as_bytes(),
                mosquitto_rs::QoS::AtLeastOnce,
                false,
            )
            .await?;

        Ok(())
    }
}

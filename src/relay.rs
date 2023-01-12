use std::{sync::Arc, time::Duration};

use futures::future::BoxFuture;
use tokio::sync::{oneshot::Receiver, RwLock};
use tracing::{debug, warn};

use crate::{
    logger::slice_to_hex,
    protocol::{Command, PFPRequest},
    protocol_parser,
    serial::SimpleSerial,
};

pub struct Relay {
    id: u32,
    serial: Arc<RwLock<SimpleSerial>>,
    on_request: Box<dyn Fn(Arc<PFPRequest>) -> BoxFuture<'static, crate::Result<()>>>,
    shutdown_signal: Receiver<()>,
    devices: Vec<()>,
}

impl Relay {
    pub fn new(
        id: u32,
        serial: Arc<RwLock<SimpleSerial>>,
        on_request: impl Fn(Arc<PFPRequest>) -> BoxFuture<'static, crate::Result<()>> + 'static,
        shutdown_signal: Receiver<()>,
    ) -> Self {
        Self {
            id,
            serial,
            on_request: Box::new(on_request),
            shutdown_signal,
            devices: Vec::new(),
        }
    }

    pub async fn run(&mut self) -> crate::Result<()> {
        let mut ttl = tokio::time::Instant::now();
        let mut timeout = tokio::time::interval(Duration::from_secs(1));

        while self.shutdown_signal.try_recv().is_err() {
            if ttl.elapsed().as_secs() >= 60 {
                ttl = tokio::time::Instant::now();
                // TODO: Check TTLs
            }

            let timeout = timeout.tick();
            tokio::pin!(timeout);
            let line = async { self.serial.write().await.read_line() };
            tokio::pin!(line);

            tokio::select! {
                _ = &mut timeout => (),
                line = &mut line => {
                    if let Ok(line) = line {
                        if let Ok((_, request)) = protocol_parser::parse(&line) {
                            let request = Arc::new(request);
                            (self.on_request)(request.clone()).await?;

                            match Command::from(request.command_id) {
                                Command::HeloP => (),
                                Command::Push => (),
                                _ => (),
                            }
                        } else {
                            warn!(
                                line = String::from_utf8(line).unwrap_or_else(|_| String::new()),
                                "Invalid request"
                            );
                        }
                    }
                }
            }

            if self.devices.is_empty() {
                let packet = Vec::from(PFPRequest::new_helop(self.id));
                debug!(packet = slice_to_hex(&packet), "Discovering devices");
                self.serial.write().await.write_buf(&packet)?;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }
}

use reqwest::Client;

pub struct Simulation {
    server: String,
    client: Client,
}

impl Simulation {
    pub fn new(server: &str) -> crate::Result<Self> {
        let client = Client::new();

        Ok(Self {
            client,
            server: server.to_string(),
        })
    }
}

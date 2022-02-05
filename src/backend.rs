use reqwest::{Client, Method, StatusCode};
use reqwest::header::{AUTHORIZATION, HeaderMap};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub url: String,
    #[serde(with = "http_serde::method")]
    pub method: Method,
    pub timeout: u8,
    #[serde(with = "http_serde::status_code")]
    pub status: StatusCode,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Incident {
    pub service: String,
    pub start: u64,
    pub end: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ping {
    pub service: String,
    pub time: u64,
    pub ms: u64,
    pub location: String,
    pub kind: Option<PingKind>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PingKind {
    INITIAL,
    ALIVE,
}

#[derive(Debug)]
pub struct Backend {
    client: Client,
    base_url: String,
    location: String,
    services: Vec<Service>,
    incidents: Vec<Incident>,
}

impl Backend {
    pub async fn new(base_url: String, token: Option<String>, location: String) -> anyhow::Result<Self> {
        let client = if let Some(token) = token {
            let mut headers = HeaderMap::new();
            headers.insert(AUTHORIZATION, format!("Bearer {token}").parse()?);

            Client::builder()
                .default_headers(headers)
                .build()?
        } else {
            Client::new()
        };

        Ok(Backend {
            client,
            base_url,
            location,
            services: Vec::new(),
            incidents: Vec::new(),
        })
    }

    pub async fn update(&mut self) -> anyhow::Result<()> {
        let (services, incidents) = tokio::try_join!(
            self.client.get(format!("{}/api/services", self.base_url)).send(),
            self.client.get(format!("{}/api/incidents?filter=active", self.base_url)).send()
        )?;

        let s = services.error_for_status()?.json().await?;
        let i = incidents.error_for_status()?.json().await?;

        self.services = s;
        self.incidents = i;

        Ok(())
    }

    pub fn location(&self) -> &String {
        &self.location
    }

    pub fn services(&self) -> &Vec<Service> {
        &self.services
    }

    pub fn incidents(&self) -> &Vec<Incident> { &self.incidents }

    pub fn find_incident(&self, service_id: &String) -> Option<&Incident> {
        for incident in &self.incidents {
            if &incident.service == service_id {
                return Some(incident);
            }
        }

        return None;
    }

    pub async fn publish_incidents(&self, incidents: &Vec<Incident>) -> anyhow::Result<()> {
        if incidents.len() == 0 {
            return Ok(());
        }

        self.client.post(format!("{}/api/incidents", self.base_url))
            .json(incidents)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn publish_pings(&self, pings: &Vec<Ping>) -> anyhow::Result<()> {
        if pings.len() == 0 {
            return Ok(());
        }

        self.client.post(format!("{}/api/pings", self.base_url))
            .json(pings)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

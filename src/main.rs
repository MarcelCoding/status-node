use std::env;
use std::time::Duration;

use chrono::Utc;
use sentry::integrations::anyhow::capture_anyhow;

use crate::backend::{Backend, Incident, Ping};
use crate::backend::PingKind::{ALIVE, INITIAL};
use crate::benchmark::execute;
use crate::cache::{Cache, FileCache};
use crate::config::Config;

mod backend;
mod benchmark;
mod cache;
mod config;

const SENTRY_PROJECT: &str = "6139029";
const SENTRY_TOKEN: &str = "be2459b57166467d9ff595ac0e0f57a9";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = sentry::init((
        format!("https://{SENTRY_TOKEN}@o1006030.ingest.sentry.io/{SENTRY_PROJECT}"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    let result = fake_main().await;

    if let Err(err) = &result {
        capture_anyhow(err);
    }

    if let Some(client) = sentry::Hub::current().client() {
        client.close(Some(Duration::from_secs(5)));
    }

    result
}

async fn fake_main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("You need to specify the path of the config file as first argument.");
        return Ok(());
    }

    let config = Config::load(&args[1]).await?;

    if config.location.is_none() {
        eprintln!("You need to configure a location.");
        return Ok(());
    }

    let cache = match config.cache {
        Some(location) => /*match parse_redis_url(&location) {
            Some(url) => Box::new(RedisCache::new(url)?) as Box<dyn Cache>,
            None => */FileCache::new(location),
        //},
        None => {
            eprintln!("You need to configure the cache.");
            return Ok(());
        }
    };

    println!("Using backend at {}.", config.backend);

    let mut backend = Backend::new(config.backend, config.token, config.location.unwrap()).await?;

    backend.update().await?;

    let mut incidents: Vec<Incident> = Vec::new();
    let mut pings: Vec<Ping> = Vec::new();

    for service in backend.services() {
        println!("Checking {} ({})...", service.id, service.namespace);
        let benchmark = execute(Duration::from_millis(500), service, 3).await?;

        if benchmark.initial.code == Some(service.status) {
            if let Some(active) = backend.find_incident(&service.namespace, &service.id) {
                incidents.push(Incident {
                    id: active.id,
                    namespace: active.namespace.clone(),
                    service: active.service.clone(),
                    start: active.start,
                    end: Some(Utc::now().timestamp() as u64),
                });
            }

            pings.push(Ping {
                namespace: service.namespace.clone(),
                service: service.id.clone(),
                time: Utc::now().timestamp() as u64,
                ms: benchmark.initial.ping,
                location: backend.location().clone(),
                kind: Some(INITIAL),
            });

            pings.push(Ping {
                namespace: service.namespace.clone(),
                service: service.id.clone(),
                time: Utc::now().timestamp() as u64,
                ms: benchmark.alive.ping,
                location: backend.location().clone(),
                kind: Some(ALIVE),
            });
        } else if backend
            .find_incident(&service.namespace, &service.id)
            .is_none()
        {
            incidents.push(Incident {
                id: None,
                namespace: service.namespace.clone(),
                service: service.id.clone(),
                start: Utc::now().timestamp() as u64,
                end: None,
            })
        }
    }

    println!();

    let known_incidents = backend.incidents();
    if known_incidents.is_empty() && incidents.is_empty() {
        println!("Currently, everything seems to be up. Good Job!")
    } else {
        println!("Currently, the following services seems to be down:");
        for incident in &incidents {
            println!(" - {} (new)", incident.service);
        }

        for incident in known_incidents {
            for new in &incidents {
                if incident.namespace == new.namespace && incident.service == new.service {
                    continue;
                }
            }
            println!(" - {}", incident.service);
        }
    }

    println!();

    backend.publish_incidents(&incidents).await?;

    let mut pings_to_publish = cache.read_if_old_enough().await?;

    if !pings_to_publish.is_empty() {
        println!("Publishing ping and truncating cache...");

        for ping in pings {
            pings_to_publish.push(ping);
        }

        backend.publish_pings(&pings_to_publish).await?;
        cache.truncate().await?;
    } else {
        cache.push(&pings).await?;
    }

    println!("Finish.");

    Ok(())
}

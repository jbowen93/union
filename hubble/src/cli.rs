use std::str::FromStr;

use clap::Parser;
use url::Url;

use crate::hasura::Datastore;

/// Hubble is state machine observer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The url to the hasura graphql endpoint.
    #[arg(short, long, env = "HUBBLE_URL")]
    pub url: Url,
    /// The admin secret used to authenticate with hasura.
    #[arg(short, long, env = "HUBBLE_SECRET")]
    pub secret: String,
    /// Indexer configurations to start.
    #[arg(short, long, env = "HUBBLE_INDEXERS")]
    pub indexers: Indexers,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Indexers(Vec<IndexerConfig>);

impl IntoIterator for Indexers {
    type Item = IndexerConfig;

    type IntoIter = std::vec::IntoIter<IndexerConfig>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum IndexerConfig {
    Tm(crate::tm::Config),
}

impl IndexerConfig {
    pub async fn index<D: Datastore>(&self, db: D) -> Result<(), color_eyre::eyre::Report> {
        match self {
            Self::Tm(cfg) => cfg.index(db).await,
        }
    }
}

impl FromStr for Indexers {
    type Err = color_eyre::eyre::Error;

    fn from_str(item: &str) -> Result<Self, <Self as FromStr>::Err> {
        serde_json::from_str(item).map_err(Into::into)
    }
}

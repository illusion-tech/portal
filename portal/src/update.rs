use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Update {
    pub html_url: String,
    pub name: String,
}

const UPDATE_URL: &str = "https://api.github.com/repos/illusion-tech/portal/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn check() {
    match check_inner().await {
        Ok(Some(new)) => {
            bunt::eprintln!(
                "{$yellow+italic}New version available:{/$} {[cyan]} => {[green]} {}",
                CURRENT_VERSION,
                new.name.as_str(),
                new.html_url
            );
        }
        Ok(None) => log::debug!("Using latest version."),
        Err(error) => log::error!("Failed to check version: {:?}", error),
    }
}

/// checks fo ra new release on github
async fn check_inner() -> Result<Option<Update>, Box<dyn std::error::Error>> {
    let update: Update = reqwest::Client::new()
        .get(UPDATE_URL)
        .header("User-Agent", "portal-client")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .json()
        .await?;

    let cur = semver::Version::from_str(CURRENT_VERSION)?;
    let remote = semver::Version::from_str(&update.name)?;

    if remote > cur {
        Ok(Some(update))
    } else {
        Ok(None)
    }
}

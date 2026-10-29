use anyhow::{Result, anyhow, bail};
use clap::Args;
use reqwest::{
    StatusCode, Url,
    blocking::{Client, RequestBuilder, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

#[derive(Args)]
pub struct ApiArgs {
    /// Base URL of the LogLumen server
    #[arg(
        long,
        env = "LOGLUMEN_URL",
        default_value = "http://localhost:3000",
        global = true
    )]
    url: Url,
    /// Admin password; read from `.env` too
    #[arg(long, env = "ADMIN_PASSWORD", hide_env_values = true, global = true)]
    admin_password: Option<String>,
}

impl ApiArgs {
    pub fn client(&self) -> Result<ApiClient> {
        if self.url.cannot_be_a_base() {
            bail!("invalid server URL: {}", self.url);
        }
        Ok(ApiClient {
            http: Client::new(),
            base: self.url.clone(),
            admin_password: self.admin_password.clone().filter(|p| !p.is_empty()),
        })
    }

    pub fn admin_password(&self) -> Option<&str> {
        self.admin_password.as_deref().filter(|p| !p.is_empty())
    }
}

pub struct ApiClient {
    http: Client,
    base: Url,
    admin_password: Option<String>,
}

impl ApiClient {
    fn url(&self, segments: &[&str]) -> Url {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .expect("base URL is checked in ApiArgs::client")
            .pop_if_empty()
            .push("api")
            .extend(segments);
        url
    }

    pub fn get<T: DeserializeOwned>(&self, segments: &[&str]) -> Result<T> {
        Ok(self.send(self.http.get(self.url(segments)))?.json()?)
    }

    pub fn post<T: DeserializeOwned>(&self, segments: &[&str], body: &impl Serialize) -> Result<T> {
        Ok(self
            .send(self.http.post(self.url(segments)).json(body))?
            .json()?)
    }

    pub fn put<T: DeserializeOwned>(&self, segments: &[&str], body: &impl Serialize) -> Result<T> {
        Ok(self
            .send(self.http.put(self.url(segments)).json(body))?
            .json()?)
    }

    pub fn delete(&self, segments: &[&str]) -> Result<()> {
        self.send(self.http.delete(self.url(segments)))?;
        Ok(())
    }

    fn send(&self, request: RequestBuilder) -> Result<Response> {
        let request = match &self.admin_password {
            Some(password) => request.basic_auth("admin", Some(password)),
            None => request,
        };
        let response = request.send()?;
        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            bail!("unauthorized: pass --admin-password or set ADMIN_PASSWORD");
        }
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            let message = match serde_json::from_str::<Value>(&body) {
                Ok(json) => json.get("error").and_then(Value::as_str).map(str::to_owned),
                Err(_) => Some(body).filter(|body| !body.trim().is_empty()),
            };
            return Err(match message {
                Some(message) => anyhow!("{status}: {message}"),
                None => anyhow!("{status}"),
            });
        }
        Ok(response)
    }
}

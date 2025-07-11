use std::collections::HashMap;

use serde::{de::DeserializeOwned, Serialize};

pub struct HttpClient {
    base_url: String,
    inner: reqwest::Client,
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            inner: reqwest::Client::new(),
        }
    }

    pub async fn get<Res: DeserializeOwned>(
        &mut self,
        url_path: &str,
        headers: Option<&HashMap<String, String>>,
        params: Option<&HashMap<String, String>>,
    ) -> anyhow::Result<Res> {
        let url = format!("{}{}", self.base_url, url_path);
        let mut request = self.inner.get(&url);

        if let Some(headers_map) = headers {
            for (key, value) in headers_map {
                request = request.header(key, value);
            }
        }

        if let Some(params_map) = params {
            for (key, value) in params_map {
                request = request.query(&[(key.as_str(), value.as_str())]);
            }
        }

        tracing::debug!("Sending GET request to {}", url);

        let response = request.send().await?;
        parse_response(response).await
    }

    pub async fn post<Req: Serialize, Res: DeserializeOwned>(
        &mut self,
        url_path: &str,
        body: &Req,
        headers: Option<&HashMap<String, String>>,
    ) -> anyhow::Result<Res> {
        let url = format!("{}{}", self.base_url, url_path);
        let mut request = self.inner.post(&url).body(serde_json::to_string(body)?);

        if let Some(headers_map) = headers {
            for (key, value) in headers_map {
                request = request.header(key, value);
            }
        }

        tracing::debug!("Sending POST request to {}", url);

        let response = request.send().await?;
        parse_response(response).await
    }
}

pub async fn parse_response<Res: DeserializeOwned>(
    response: reqwest::Response,
) -> anyhow::Result<Res> {
    let status = response.status().as_u16();
    let text = response.text().await?;
    tracing::debug!("Response status: {}, body: {}", status, text);

    if (200..300).contains(&status) {
        serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))
    } else {
        Err(anyhow::anyhow!(
            "Request failed with status {} and body: {}",
            status,
            text
        ))
    }
}

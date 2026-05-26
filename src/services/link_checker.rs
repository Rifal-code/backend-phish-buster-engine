use std::time::Duration;
use reqwest::Client; // Memperbaiki typo 'use use_reqwest::Client'
use serde_json::Value;

pub struct LinkCheckerService {
    client: Client,
    api_key: String,
    api_url: String,
}

impl LinkCheckerService {
    pub fn new(api_key: String, api_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .expect("Failed to build HTTP client");

        Self { client, api_key, api_url }
    }

    /// Call the Gemini API and parse its response into a flat analysis object.
    /// Returns a JSON value with at minimum: { status, score, reason }.
    pub async fn check_url(&self, url: &str) -> Result<Value, LinkCheckerError> {
        if self.api_url.is_empty() || self.api_key.is_empty() {
            return Err(LinkCheckerError::NotConfigured);
        }

        let prompt = format!(
            "You are a cybersecurity URL analysis engine. \
            Analyze the following URL for threats such as phishing, scam, malware, judol (illegal gambling), or other malicious activity. \
            \n\n\
            CRITICAL RULES:\n\
            1. 'status' MUST be exactly one of these values: \"safe\", \"suspicious\", \"malicious\", or \"judol\".\n\
            2. 'score' MUST be an integer between 0 and 100. DO NOT use null. If the URL is clean/safe, give it a score of 0.\n\
            3. 'reason' MUST be a brief string explaining the judgment.\n\n\
            URL: {}",
            url
        );

        let endpoint = format!("{}?key={}", self.api_url, self.api_key);

        let response = self
            .client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "contents": [{
                    "parts": [{ "text": prompt }]
                }],
                "generationConfig": {
                    "temperature": 0.0,
                    "maxOutputTokens": 256,
                    "responseMimeType": "application/json"
                }
            }))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LinkCheckerError::Timeout
                } else {
                    LinkCheckerError::RequestFailed(e.to_string())
                }
            })?;

        let http_status = response.status();
        if !http_status.is_success() {
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Gemini API error {}: {}", http_status, body);
            return Err(LinkCheckerError::ApiError(format!(
                "AI API returned status {}",
                http_status.as_u16()
            )));
        }

        let raw: Value = response
            .json()
            .await
            .map_err(|e| LinkCheckerError::ParseError(e.to_string()))?;

        let text = raw
            .pointer("/candidates/0/content/parts/0/text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        let parsed: Value = serde_json::from_str(&text).unwrap_or_else(|_| {
            tracing::warn!("Gemini returned non-JSON text unexpectedly: {}", text);
            serde_json::json!({
                "status": "unknown",
                "score": 0,
                "reason": text
            })
        });

        Ok(parsed)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LinkCheckerError {
    #[error("Link checker API is not configured")]
    NotConfigured,

    #[error("Request timed out")]
    Timeout,

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

use std::path::Path;

use reqwest::{Client, multipart};
use serde_json::Value;
use tokio::{process::Command, sync::Semaphore};

use super::general;
use crate::AppError;

const USER_AGENT: &str = "Buh/1.0";
const MEDIAWIKI_CLIENT: &str = "/home/thegrieferknight/Projects/wiki-tools/mediawiki-client";

/// Uploads are throttled separately from set processing, since several sets
/// can be uploading icons at the same time.
const MAX_CONCURRENT_UPLOADS: usize = 5;
static UPLOAD_PERMITS: Semaphore = Semaphore::const_new(MAX_CONCURRENT_UPLOADS);

pub struct Wiki {
    api_url: String,
    client: Client,
    csrf_token: String,
    user: String,
    pass: String,
}

impl Wiki {
    pub async fn login(api_url: &str, user: String, pass: String) -> Result<Self, AppError> {
        // The cookie store keeps the session between requests.
        let client = Client::builder()
            .cookie_store(true)
            .user_agent(USER_AGENT)
            .build()?;

        let login_token = fetch_token(&client, api_url, "login").await?;

        let r: Value = client
            .post(api_url)
            .form(&[
                ("action", "login"),
                ("lgname", &user),
                ("lgpassword", &pass),
                ("lgtoken", &login_token),
                ("format", "json"),
            ])
            .send()
            .await?
            .json()
            .await?;
        println!("Login: {}", r["login"]["result"]);

        let csrf_token = fetch_token(&client, api_url, "csrf").await?;

        Ok(Self {
            api_url: api_url.to_owned(),
            client,
            csrf_token,
            user,
            pass,
        })
    }

    /// Returns the raw wikitext of `title` via the external mediawiki-client.
    pub async fn read_page(&self, title: &str) -> Result<Vec<u8>, String> {
        let output = Command::new(MEDIAWIKI_CLIENT)
            .arg("read")
            .arg(title)
            .env("WIKI_API_URL", &self.api_url)
            .env("WIKI_BOT_USER", &self.user)
            .env("WIKI_BOT_PASS", &self.pass)
            .output()
            .await
            .map_err(|e| format!("failed to run mediawiki-client: {e}"))?;

        if !output.status.success() {
            return Err(format!(
                "mediawiki-client read failed ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(output.stdout)
    }

    /// Uploads `file` as `filename`, overwriting any existing file.
    pub async fn upload_file(
        &self,
        filename: &str,
        comment: String,
        file: impl AsRef<Path>,
    ) -> Result<(), AppError> {
        let form = multipart::Form::new()
            .text("action", "upload")
            .text("filename", filename.to_owned())
            .text("comment", comment)
            .text("text", "")
            .text("token", self.csrf_token.clone())
            .text("ignorewarnings", "1")
            .text("format", "json")
            .file("file", file)
            .await?;

        let _permit = UPLOAD_PERMITS
            .acquire()
            .await
            .expect("upload semaphore closed");

        let r: Value = self
            .client
            .post(&self.api_url)
            .multipart(form)
            .send()
            .await?
            .json()
            .await?;

        println!("Upload result: {}", r["upload"]["result"]);
        println!("{}", serde_json::to_string_pretty(&r)?);
        Ok(())
    }
}

async fn fetch_token(client: &Client, api_url: &str, kind: &str) -> Result<String, AppError> {
    let r: Value = client
        .get(api_url)
        .query(&[
            ("action", "query"),
            ("meta", "tokens"),
            ("type", kind),
            ("format", "json"),
        ])
        .send()
        .await?
        .json()
        .await?;

    let key = format!("{kind}token");
    r["query"]["tokens"][&key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| general(format!("missing {key} in response: {r}")))
}

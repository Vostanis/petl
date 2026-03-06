use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures::stream::StreamExt;
use tokio::{fs, io::AsyncWriteExt};
use tracing::{error, info, trace};

#[async_trait]
pub trait HttpExtractExt {
    async fn download_file(&self, url: &str, path: &str) -> Result<()>;
}

#[async_trait]
impl HttpExtractExt for reqwest::Client {
    async fn download_file(&self, url: &str, path: &str) -> Result<()> {
        // Make HTTP GET request
        let response = self.get(url).send().await.map_err(|e| {
            error!(url=%url, path=%path, "Failed to download file; {e}");
            e
        })?;

        info!(url=%url, "{status}", status=response.status());

        let dir = std::path::Path::new(path);
        fs::create_dir_all(dir.parent().expect("failed to retrieve parent directory")).await?;

        // Fail if reponse status code is not between 200-299
        if !response.status().is_success() {
            error!(url=%url, path=%path, "HTTP response status: {}", response.status());
            return Ok(());
        }
        trace!(
            url=%url,
            "Request successful - HTTP response status: {}",
            response.status()
        );

        let content_length = response.content_length();

        // Create an empty file to download to
        let temp_path = format!("{path}.tmp");
        let mut file = fs::File::create(&temp_path).await.map_err(|e| {
            error!(url=%url, "Failed to create file at {path}; {e}");
            e
        })?;
        trace!(url=%url, "Empty file created at {path}");

        // Stream the bytes, async, writing into the file
        let mut stream = response.bytes_stream();
        let mut downloaded = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
        }
        file.flush().await?;
        drop(file);

        // Handle corrupted files
        if let Some(expected) = content_length {
            if downloaded != expected {
                fs::remove_file(&temp_path).await?;
                error!(url=%url, expected=%expected, downloaded=%downloaded, "incomplete download - size mismatch");
                return Err(anyhow!(
                    "incomplete download: expected {expected} - downloaded {downloaded}"
                ));
            }
        }

        fs::rename(&temp_path, path).await.map_err(|e| {
            error!(url=%url, "failed to rename temporary file {temp_path} to {path}; {e}");
            e
        })?;

        trace!(url=%url, "File written to {path}");

        Ok(())
    }
}

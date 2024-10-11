use anyhow::Context;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use log::{log, Level};
use rocket::fairing::AdHoc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Clone)]
pub struct Downloader {
    client: Client,
    bucket: String,
    local_dir: String,
}

impl Downloader {
    pub async fn clean_download(&self) -> Result<(), anyhow::Error> {
        self.clear_dir()?;
        self.download_directory(self.bucket.as_str(), self.local_dir.as_str()).await
    }

    async fn download_directory(&self, bucket: &str, local_dir: &str) -> Result<(), anyhow::Error> {
        let list_objects = self.client.list_objects_v2()
            .bucket(bucket)
            // .prefix(prefix)
            .send()
            .await?;

        if let Some(contents) = list_objects.contents {
            for object in contents {
                if let Some(key) = object.key {
                    let get_object = self.client.get_object()
                        .bucket(bucket)
                        .key(&key)
                        .send()
                        .await
                        .context("send get object request")?;
                    let body = get_object
                        .body
                        .collect()
                        .await
                        .context("collect body")?;
                    let local_path = format!("{}/{}", local_dir.replace("/", "\\"), key);
                    log!(Level::Info, "Downloading {} to {}", key, local_path);
                    //create subdirectories if they don't exist
                    let local_path = std::path::Path::new(&local_path);
                    if let Some(parent) = local_path.parent() {
                        std::fs::create_dir_all(parent).context("create directory")?;
                    }
                    let mut file = File::create(local_path).await.context("create file")?;
                    file.write_all(&body.into_bytes()).await.context("write to file")?;
                }
            }
        }

        Ok(())
    }

    pub fn manage(downloader: Downloader) -> AdHoc {
        AdHoc::on_ignite("S3 Downloader", move |rocket| async move {
            // let downloader = Downloader::new().await;
            rocket.manage(downloader)
        })
    }

    pub fn managed(bucket: String, local_dir: String) -> AdHoc {
        AdHoc::on_ignite("S3 Downloader", move |rocket| async move {
            let downloader = Downloader::new(bucket.as_str(), local_dir.as_str()).await;
            log!(Level::Info, "Start downloading bucket {} to {}", bucket, local_dir);
            downloader.clean_download().await.expect("Failed to download files");
            rocket.manage(downloader)
        })
    }

    // AWS_ENDPOINT_URL_S3
    // AWS_ACCESS_KEY_ID
    // AWS_SECRET_ACCESS_KEY
    // AWS_REGION
    pub async fn new(bucket: &str, local_dir: &str) -> Self {
        let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            // .endpoint_url("https://innovations-cloud.avlab.dev")
            .load()
            .await;
        let client = Client::new(&config);
        log!(Level::Info, "Connected to S3");
        Self { client, bucket: bucket.to_string(), local_dir: local_dir.to_string() }
    }

    //remove all files and subdirs in directory local_dir
    fn clear_dir(&self) -> Result<(), anyhow::Error> {
        let local_dir = std::path::Path::new(&self.local_dir);
        if local_dir.exists() {
            for entry in std::fs::read_dir(local_dir).context("read directory")? {
                let entry = entry.context("entry")?;
                let path = entry.path();
                if path.is_dir() {
                    log!(Level::Info, "Removing directory {}", path.display());
                    std::fs::remove_dir_all(&path).context("remove directory")?;
                } else {
                    log!(Level::Info, "Removing file {}", path.display());
                    std::fs::remove_file(&path).context("remove file")?;
                }
            }
        }
        Ok(())
    }
}
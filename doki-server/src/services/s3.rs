use anyhow::Context;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub async fn download_directory(&self, bucket: &str, local_dir: &str) -> Result<(), anyhow::Error> {
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
                    println!("Downloading {} to {}", key, local_path);
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

    // AWS_ENDPOINT_URL_S3
    // AWS_ACCESS_KEY_ID
    // AWS_SECRET_ACCESS_KEY
    // AWS_REGION
    pub async fn new() -> Self {
        let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            // .endpoint_url("https://innovations-cloud.avlab.dev")
            .load()
            .await;
        let client = Client::new(&config);
        Self { client }
    }
}
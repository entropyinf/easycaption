use crate::Res;
use anyhow::bail;
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::fs;
use std::io::{BufWriter, Seek, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

const FILES_URL: &str = "https://modelscope.cn/api/v1/models/<model_id>/repo/files?Recursive=true";
const DOWNLOAD_URL: &str = "https://modelscope.cn/models/<model_id>/resolve/master/<path>";

const UA: (&str, &str) = (
    "User-Agent",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36",
);

#[derive(Debug, Deserialize)]
struct ModelScopeResponse {
    #[serde(rename = "Code")]
    code: i32,
    #[serde(rename = "Data")]
    data: ModelScopeResponseData,
}

#[derive(Debug, Deserialize)]
struct ModelScopeResponseData {
    #[serde(rename = "Files")]
    files: Vec<RepoFile>,
}

#[derive(Debug, Deserialize, Clone)]
struct RepoFile {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Path")]
    path: String,
    #[serde(rename = "Size")]
    size: u64,
    #[serde(rename = "Sha256")]
    #[allow(unused)]
    sha256: String,
}

const BAR_STYLE: &str = "{msg:<30} {bar} {decimal_bytes:<10} / {decimal_total_bytes:<10} {decimal_bytes_per_sec:<12} {percent:<3}%  {eta_precise}";

pub struct ModelScopeRepo {
    model_id: String,
    save_dir: PathBuf,
    repo_files: RwLock<Option<Vec<RepoFile>>>,
}

impl ModelScopeRepo {
    pub fn new<P: AsRef<Path>>(model_id: &str, save_dir: P) -> Self {
        let save_dir = save_dir.as_ref().join(model_id);
        ModelScopeRepo {
            model_id: model_id.to_string(),
            save_dir,
            repo_files: RwLock::new(None),
        }
    }

    pub async fn get(&self, file: &str) -> Res<PathBuf> {
        let file_path = self.save_dir.join(file);
        if file_path.exists() {
            return Ok(file_path);
        }

        self.download(file).await?;

        Ok(file_path)
    }

    async fn download(&self, file: &str) -> Res<()> {
        info!(
            "Downloading model {} to: {}",
            self.model_id,
            self.save_dir.display()
        );

        fs::create_dir_all(&self.save_dir)?;

        let repo_files = self.get_repo_files().await?;

        if let Some(repo_file) = repo_files.into_iter().find(|f| f.name == file) {
            let bar = ProgressBar::new(repo_file.size);
            let style = ProgressStyle::default_bar().template(BAR_STYLE)?;
            bar.set_style(style);
            self.download_file(repo_file, bar).await?;
        }

        Ok(())
    }

    async fn get_repo_files(&self) -> Res<Vec<RepoFile>> {
        {
            let cached_repo_files = self.repo_files.read().await;
            if cached_repo_files.is_some() {
                return Ok(cached_repo_files.clone().unwrap());
            };
        }

        let files_url = FILES_URL.replace("<model_id>", &self.model_id);
        let resp = reqwest::get(files_url).await?;
        let response = resp.json::<ModelScopeResponse>().await?;
        if response.code != 200 {
            bail!("Failed to get model files: {}", response.code);
        }
        let repo_files = response.data.files;

        self.repo_files.write().await.replace(repo_files.clone());

        Ok(repo_files)
    }

    async fn download_file(&self, repo_file: RepoFile, bar: ProgressBar) -> Res<()> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .build()?;

        let path = &repo_file.path;
        let name = &repo_file.name;

        bar.set_message(name.clone());
        let temp_file_path = self.save_dir.join(format!("{path}.downloading"));

        if let Some(parent) = temp_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut existing_size = 0;
        let mut file_options = fs::OpenOptions::new();
        file_options.write(true).create(true);

        if temp_file_path.exists() {
            if let Ok(metadata) = fs::metadata(&temp_file_path) {
                existing_size = metadata.len();
                file_options.append(true);
            }
        } else {
            file_options.truncate(true);
        }

        let mut file = BufWriter::new(file_options.open(&temp_file_path)?);

        // Set progress bar initial position
        bar.set_position(existing_size);
        bar.set_length(repo_file.size);

        let url = DOWNLOAD_URL
            .replace("<model_id>", &self.model_id)
            .replace("<path>", path);

        let mut rb = client.get(&url).header(UA.0, UA.1);

        // Already downloaded, just return ok.
        // If file size equal repo file size, maybe check sha256
        // But I think the probability of files having the same number of bytes is relatively low, so I won't check here. 🙊
        if existing_size == repo_file.size {
            bar.finish();
            let file_path = self.save_dir.join(path);
            fs::rename(&temp_file_path, &file_path)?;
            return Ok(());
        }

        // Resume download
        if existing_size < repo_file.size {
            rb = rb.header("Range", format!("bytes={}-", existing_size));
        }

        let response = rb.send().await?;
        let status = response.status();

        // Server doesn't support resume download, re-downloading from beginning
        // Or existing file size is larger than repo size, re-downloading from beginning
        if status == reqwest::StatusCode::OK && existing_size > 0 || existing_size > repo_file.size
        {
            file.rewind()?;
            file.get_ref().set_len(0)?;
            bar.set_position(0);
        }

        // If status is not success or partial content, bail
        if !response.status().is_success()
            && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
        {
            bail!(
                "Failed to download file {}: HTTP {}",
                name,
                response.status()
            );
        }

        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk)?;
            bar.inc(chunk.len() as u64);
        }

        file.flush()?;

        let file_path = self.save_dir.join(path);
        fs::rename(&temp_file_path, &file_path)?;
        bar.finish();

        Ok(())
    }
}

trait ProgressView {}

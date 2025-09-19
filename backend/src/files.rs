use crate::models::file::File;
use chrono::NaiveDateTime;
use colored::Colorize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicI32, AtomicI64},
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{Mutex, RwLock},
};

struct CachedFile {
    id: i32,
    size: i32,

    last_access: NaiveDateTime,
    last_access_written: bool,
}

type CachedFiles = HashMap<Arc<PathBuf>, Arc<Mutex<CachedFile>>>;
pub struct FileCache {
    id: Arc<AtomicI32>,
    total_size: Arc<AtomicI64>,
    max_cache_size: i64,
    cached_files: Arc<RwLock<CachedFiles>>,

    database: Arc<crate::database::Database>,
    env: Arc<crate::env::Env>,
}

impl FileCache {
    pub async fn new(database: Arc<crate::database::Database>, env: Arc<crate::env::Env>) -> Self {
        tokio::fs::remove_dir_all(&env.files_cache)
            .await
            .unwrap_or_default();
        tokio::fs::create_dir_all(&env.files_cache).await.unwrap();

        Self {
            id: Arc::new(AtomicI32::new(0)),
            total_size: Arc::new(AtomicI64::new(0)),
            max_cache_size: 5 * 1024 * 1024 * 1024,
            cached_files: Arc::new(RwLock::new(HashMap::new())),
            database,
            env,
        }
    }

    pub async fn get(
        &self,
        path: &Path,
        file: &File,
    ) -> std::io::Result<Box<dyn tokio::io::AsyncRead + Send + Unpin>> {
        if file.is_directory {
            return Err(std::io::Error::other("Cannot get file for directory"));
        }

        let key = Arc::new(path.to_path_buf());
        let cached_files = self.cached_files.read().await;

        if let Some(cached_file) = cached_files.get(&key) {
            let mut cached_file = cached_file.lock().await;
            cached_file.last_access = chrono::Utc::now().naive_utc();
            cached_file.last_access_written = false;

            return Ok(Box::new(
                tokio::fs::File::open(
                    Path::new(&self.env.files_cache).join(cached_file.id.to_string()),
                )
                .await?,
            ));
        }

        drop(cached_files);

        let file_size = file.size;

        if self.total_size.load(std::sync::atomic::Ordering::SeqCst) + file_size
            > self.max_cache_size
        {
            self.make_space_for_file(file_size).await?;
        }

        let cached_file = Arc::new(Mutex::new(CachedFile {
            id: self.id.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            size: file.size as i32,
            last_access: chrono::Utc::now().naive_utc(),
            last_access_written: false,
        }));
        let cached_file_lock = cached_file.lock().await;

        self.cached_files
            .write()
            .await
            .insert(key.clone(), cached_file.clone());

        let source = Path::new(&self.env.files_location).join(path);
        let destination = Path::new(&self.env.files_cache).join(cached_file_lock.id.to_string());

        let (return_reader, mut return_writer) = tokio::io::duplex(32 * 1024);
        let (file_allow_sender, mut file_allow_reader) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn({
            let destination = destination.clone();

            async move {
                let mut skip_allow_check = false;
                file_allow_reader.recv().await;

                let mut run = async || -> std::io::Result<()> {
                    let mut buffer = vec![0; 32 * 1024];
                    let mut file = tokio::fs::File::open(&destination).await?;

                    loop {
                        match file.read(&mut buffer).await? {
                            0 => break,
                            n => return_writer.write_all(&buffer[..n]).await?,
                        }

                        if !skip_allow_check && file_allow_reader.recv().await == Some(true) {
                            skip_allow_check = true;
                        }
                    }

                    Ok(())
                };

                if run().await.is_err() {
                    file_allow_reader.close();
                }
            }
        });

        tokio::spawn({
            drop(cached_file_lock);

            let cached_files = self.cached_files.clone();
            let total_size = self.total_size.clone();

            async move {
                let cached_file_lock = cached_file.lock().await;

                let run = async || -> std::io::Result<()> {
                    let mut buffer = vec![0; 32 * 1024];
                    let mut reader = tokio::fs::File::open(source).await?;
                    let mut file = tokio::fs::File::create(&destination).await?;

                    loop {
                        match reader.read(&mut buffer).await? {
                            0 => {
                                file.sync_all().await?;
                                break;
                            }
                            n => {
                                file.write_all(&buffer[..n]).await?;
                                file.flush().await?;
                                file_allow_sender.send(false).ok();
                            }
                        }
                    }

                    file_allow_sender.send(true).ok();

                    Ok(())
                };

                if let Err(err) = run().await {
                    cached_files.write().await.remove(&key);
                    tokio::fs::remove_file(destination)
                        .await
                        .unwrap_or_default();

                    panic!("{:?}", err);
                }

                total_size.fetch_add(
                    cached_file_lock.size as i64,
                    std::sync::atomic::Ordering::SeqCst,
                );
            }
        });

        Ok(Box::new(return_reader))
    }

    async fn make_space_for_file(&self, required_size: i64) -> std::io::Result<()> {
        let mut cached_files_lock = self.cached_files.write().await;

        if required_size > self.max_cache_size {
            return Err(std::io::Error::other(format!(
                "file size {} exceeds maximum cache size {}",
                required_size, self.max_cache_size
            )));
        }

        let mut files_with_access_time = cached_files_lock
            .iter()
            .filter_map(|(path, file)| {
                if let Ok(file_lock) = file.try_lock() {
                    Some((
                        path.clone(),
                        file.clone(),
                        file_lock.last_access,
                        file_lock.id,
                        file_lock.size as i64,
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        files_with_access_time.sort_by(|a, b| a.2.cmp(&b.2));

        let current_size = self.total_size.load(std::sync::atomic::Ordering::SeqCst);
        let target_size = current_size + required_size - self.max_cache_size;

        crate::logger::log(
            crate::logger::LoggerLevel::Info,
            format!(
                "cache size: {}/{} bytes, need to free {} bytes",
                current_size.to_string().cyan(),
                self.max_cache_size.to_string().cyan(),
                target_size.to_string().cyan()
            ),
        );

        let mut freed_size = 0;
        let mut removed_count = 0;

        for (path, _, _, id, size) in files_with_access_time {
            if freed_size >= target_size {
                break;
            }

            match tokio::fs::remove_file(Path::new(&self.env.files_cache).join(id.to_string()))
                .await
            {
                Ok(_) => {
                    freed_size += size;
                    removed_count += 1;

                    cached_files_lock.remove(&path);

                    self.total_size
                        .fetch_sub(size, std::sync::atomic::Ordering::SeqCst);

                    crate::logger::log(
                        crate::logger::LoggerLevel::Info,
                        format!(
                            "removed file {} from cache, freed {} bytes",
                            path.display().to_string().yellow(),
                            size.to_string().cyan()
                        ),
                    );
                }
                Err(e) => {
                    crate::logger::log(
                        crate::logger::LoggerLevel::Error,
                        format!(
                            "failed to remove file {} from cache: {}",
                            path.display().to_string().red(),
                            e
                        ),
                    );
                }
            }
        }

        crate::logger::log(
            crate::logger::LoggerLevel::Info,
            format!(
                "freed {} bytes by removing {} files from cache",
                freed_size.to_string().cyan(),
                removed_count.to_string().cyan()
            ),
        );

        if freed_size < target_size {
            return Err(std::io::Error::other(format!(
                "could not free enough space in cache. needed {} bytes, freed {} bytes",
                target_size, freed_size
            )));
        }

        Ok(())
    }

    pub async fn process(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pending_files = self
            .cached_files
            .read()
            .await
            .iter()
            .map(|(k, f)| (k, f.try_lock()))
            .filter(|(_, f)| f.as_ref().is_ok_and(|f| !f.last_access_written))
            .map(|(k, f)| (k.clone(), f.unwrap().last_access))
            .collect::<Vec<_>>();

        let pending_files_len = pending_files.len();
        for (path, last_access) in pending_files {
            if let Some(entry) = self.cached_files.read().await.get(&path) {
                entry.lock().await.last_access_written = true;
            }

            match sqlx::query!(
                r#"
                UPDATE files
                SET last_access = $1
                WHERE files.path = $2::varchar[] AND (files.last_access IS NULL OR files.last_access < $1)
                "#,
                Some(last_access),
                &path
                    .components()
                    .filter(|c| c.as_os_str().to_str().is_some_and(|s| !s.is_empty()))
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
            )
            .execute(self.database.write())
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    crate::logger::log(
                        crate::logger::LoggerLevel::Error,
                        format!("{} {}", "failed to update file".red(), e),
                    );

                    return Err(Box::new(e));
                }
            }
        }

        if pending_files_len > 0 {
            crate::logger::log(
                crate::logger::LoggerLevel::Info,
                format!(
                    "processed {} pending files",
                    pending_files_len.to_string().cyan()
                ),
            );
        }

        let deletion_threshold =
            chrono::Utc::now().naive_utc() - std::time::Duration::from_secs(24 * 60 * 60);
        let mut deletable_files_lock = self.cached_files.write().await;
        let deletable_files = deletable_files_lock
            .iter()
            .map(|(k, f)| (k, f.try_lock()))
            .filter(|(_, f)| f.as_ref().is_ok_and(|f| f.last_access < deletion_threshold))
            .map(|(k, f)| (k.clone(), (f.as_ref().unwrap().id, f.unwrap().size)))
            .collect::<Vec<_>>();

        for (path, (id, size)) in deletable_files.into_iter() {
            tokio::fs::remove_file(Path::new(&self.env.files_cache).join(id.to_string())).await?;
            self.total_size
                .fetch_sub(size as i64, std::sync::atomic::Ordering::SeqCst);
            deletable_files_lock.remove(&path);
        }

        Ok(())
    }
}

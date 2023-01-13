use std::env;
use std::fs::File;
use std::io::Write;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;

use crate::core::HTTP_CLIENT;

pub async fn install(
    config: super::InstallConfig,
    callback: impl Fn(usize) + Send + 'static,
) -> crate::Result<()> {
    let mut target = File::create(Path::new(&config.install_root).join("limit-server.Appimage"))?;

    let mut permission = target.metadata()?.permissions();
    permission.set_mode(0o755);
    target.set_permissions(permission)?;

    let mut resp = HTTP_CLIENT
        .get(format!(
            "https://github.com/Limit-LAB/limit-server/releases/latest/download/limit_up-{}-{}-{}",
            env!("TARGET_ARCH"),
            env!("TARGET_OS"),
            env!("TARGET_ENV")
        ))
        .send()
        .await?;

    let total = resp
        .content_length()
        .ok_or("Unknown size when downloading Appimage".to_string())?;
    let mut current = 0;
    let mut old_progress = 0;

    while let Some(chunk) = resp.chunk().await? {
        target.write_all(&chunk)?;

        current += chunk.len() as u64;
        let new_progress = (current as f64 / total as f64 * 100.0) as usize;
        if new_progress != old_progress {
            callback(new_progress);
            old_progress = new_progress;
        }
    }

    Ok(())
}

pub async fn update() {}

pub async fn uninstall() {}

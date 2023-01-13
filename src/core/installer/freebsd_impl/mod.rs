mod pkgmanager;

use super::{find_command, InstallConfig};
use r18::tr;
use std::{
    iter::empty,
    path::Path,
    process::{ExitStatus, Stdio}, sync::Arc,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    select,
};

use crate::core::{helper::Help, installer::freebsd_impl::pkgmanager::PackageManager};

pub async fn install(
    config: InstallConfig,
    callback: impl Fn(usize, String, String) + Send + 'static,
) -> crate::Result<()> {
    let callback = Arc::new(callback);
    let mut progress = 0;

    // install Elixir
    if find_command("iex", empty::<&str>()).is_empty() {
        install_elixir(&mut progress, callback.clone()).await?;
    }

    progress = 50;
    callback(progress, String::new(), String::new());

    // install or update the server repo
    clone_or_pull_repo(config, &mut progress, callback.clone()).await
}

async fn trace_process(
    mut proc: Child,
    progress: &mut usize,
    max_progress: usize,
    callback: Arc<impl Fn(usize, String, String) + Send + 'static>,
    on_failed: impl FnOnce(ExitStatus) -> crate::Error,
) -> crate::Result<()> {
    let mut stdout = BufReader::new(proc.stdout.take().unwrap());
    let mut stderr = BufReader::new(proc.stderr.take().unwrap());

    loop {
        let mut out_buf = String::new();
        let mut err_buf = String::new();

        select! {
            _ = stdout.read_line(&mut out_buf) => {
                if *progress < max_progress - 1 {
                    *progress += 1;
                }

                callback(*progress, out_buf, String::new());
            },
            _ = stderr.read_line(&mut err_buf) => {
                callback(*progress, String::new(), err_buf);
            },
            status = proc.wait() => {
                let status = status?;
                if status.success() {
                    break Ok(());
                }

                return Err(on_failed(status));
            }
        }
    }
}

async fn install_elixir(
    progress: &mut usize,
    callback: Arc<impl Fn(usize, String, String) + Send + 'static>,
) -> crate::Result<()> {
    let proc = PackageManager::new()?.install(vec!["elixir"]).await?;

    trace_process(proc, progress, 49, callback, |e| {
        tr!(
            "Package manager exit with {}\n\n{}",
            e.to_string(),
            Help::Network.to_string()
        )
        .into()
    })
    .await
}

async fn clone_or_pull_repo(
    config: InstallConfig,
    progress: &mut usize,
    callback: Arc<impl Fn(usize, String, String) + Send + 'static>,
) -> crate::Result<()> {
    let mut command = Command::new("git");

    match Path::new(&config.install_root)
        .join("limit-server")
        .exists()
    {
        // pull limit-server repo
        true => command.args(&["pull", "--recurse-submodules"]),
        // clone limit-server repo
        false => command.args(&[
            "clone",
            "--recursive",
            "https://github.com/Limit-LAB/limit-server",
        ]),
    };

    let proc = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    trace_process(proc, progress, 99, callback, |e| {
        tr!(
            "Package manager exit with {}\n\n{}",
            e.to_string(),
            Help::Network.to_string()
        )
        .into()
    })
    .await
}

pub async fn update() {}

pub async fn uninstall() {}

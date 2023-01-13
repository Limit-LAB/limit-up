pub async fn install(
    config: super::InstallConfig,
    callback: impl Fn(usize) + Send + 'static,
) -> crate::Result<()> {
    Err("Unsupported platform".into())
}

pub async fn update() {}

pub async fn uninstall() {}

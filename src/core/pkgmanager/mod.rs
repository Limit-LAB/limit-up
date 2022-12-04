use std::{
    process::{Child, Command},
    sync::Arc,
};

use once_cell::sync::Lazy;

trait PkgManager {
    fn install(&self, pkg: &str) -> std::io::Result<Child>;
    fn uninstall(&self, pkg: &str) -> std::io::Result<Child>;
    fn name(&self) -> &'static str;
}

macro_rules! impl_pkg_manager {
    ($class:ident, $name:expr, $install:expr, $uninstall:expr, $($flag:expr),*) => {
        pub struct $class;

        impl PkgManager for $class {
            fn install(&self, pkg: &str) -> std::io::Result<Child> {
                Command::new($name)
                    .args(vec![$install.into(), pkg, $($flag),*])
                    .spawn()
            }

            fn uninstall(&self, pkg: &str) -> std::io::Result<Child> {
                Command::new($name)
                    .args(vec![$uninstall.into(), pkg, $($flag),*])
                    .spawn()
            }

            fn name(&self) -> &'static str {
                $name
            }
        }
    };
}

#[cfg(target_family = "unix")]
impl_pkg_manager!(Apt, "apt", "install", "remove", "-y");
#[cfg(target_family = "unix")]
impl_pkg_manager!(Dnf, "dnf", "install", "remove", "-y");
#[cfg(target_family = "unix")]
impl_pkg_manager!(Pacman, "pacman", "-S", "-Rns", "--noconfirm");
#[cfg(target_family = "unix")]
impl_pkg_manager!(Zypper, "zypper", "install", "remove", "-y");

#[derive(Debug, Clone)]
pub enum Error {
    IoError(Arc<std::io::Error>),
    PermissionDenied,
    NotSupported,
}

macro_rules! boxed_mgrs {
    ($($mgr:ident),+) => {
        vec![$(Box::new($mgr {})),+]
    };
}

type PkgManagerFactory = Result<Box<dyn PkgManager + Send + Sync>, Error>;

static PKG_MANAGER: Lazy<PkgManagerFactory> = Lazy::new(init_pkg_manager);

fn init_pkg_manager() -> PkgManagerFactory {
    #[cfg(target_family = "unix")]
    let pkgs: Vec<Box<dyn PkgManager + Send + Sync>> = boxed_mgrs![Apt, Dnf, Pacman, Zypper];

    #[cfg(target_family = "unix")]
    return pkgs
        .into_iter()
        .find(|pkg| Command::new(pkg.name()).output().is_ok())
        .ok_or(Error::NotSupported)
        .and_then(|pkg| match unsafe { libc::geteuid() } {
            0 => Ok(pkg),
            _ => Err(Error::PermissionDenied),
        });

    #[cfg(not(target_family = "unix"))]
    Err(Error::NotSupported)
}

pub fn install(pkg: impl AsRef<str>) -> Result<Child, Error> {
    PKG_MANAGER.as_ref().map_err(|e| e.clone()).and_then(|mgr| {
        mgr.install(pkg.as_ref())
            .map_err(|e| Error::IoError(Arc::new(e)))
    })
}

pub fn uninstall(pkg: impl AsRef<str>) -> Result<Child, Error> {
    PKG_MANAGER.as_ref().map_err(|e| e.clone()).and_then(|mgr| {
        mgr.uninstall(pkg.as_ref())
            .map_err(|e| Error::IoError(Arc::new(e)))
    })
}

pub fn name() -> Result<&'static str, Error> {
    PKG_MANAGER
        .as_ref()
        .map(|mgr| mgr.name())
        .map_err(|e| e.clone())
}

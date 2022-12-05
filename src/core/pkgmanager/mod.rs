use std::{
    process::{Child, Command, Stdio},
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
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
            }

            fn uninstall(&self, pkg: &str) -> std::io::Result<Child> {
                Command::new($name)
                    .args(vec![$uninstall.into(), pkg, $($flag),*])
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
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

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::IoError(ref e) => write!(f, "I/O error: {}", e),
            Error::PermissionDenied => write!(
                f,
                "Permission denied, please rerun as root or administrator"
            ),
            Error::NotSupported => write!(f, "Unsupported package manager or platform"),
        }
    }
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
    let mgrs: Vec<Box<dyn PkgManager + Send + Sync>> = boxed_mgrs![Apt, Dnf, Pacman, Zypper];

    #[cfg(target_family = "unix")]
    return mgrs
        .into_iter()
        .find(|mgr| Command::new(mgr.name()).output().is_ok())
        .ok_or(Error::NotSupported)
        .and_then(|mgr| match unsafe { libc::geteuid() } {
            0 => Ok(mgr),
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

// Please run the test as administrator serial
#[cfg(test)]
mod tests {    
    #[test]
    fn name() {
        println!("package manager: {}", super::name().unwrap());
    }

    #[test]
    fn install() {
        let res = super::install("cowsay")
            .unwrap()
            .wait_with_output()
            .unwrap();

        println!("{}", res.status);
        println!("stdout:\n {}\n", String::from_utf8(res.stdout).unwrap());
        println!("stderr:\n {}\n", String::from_utf8(res.stderr).unwrap());
    }

    #[test]
    fn uninstall() {
        let res = super::uninstall("cowsay")
            .unwrap()
            .wait_with_output()
            .unwrap();

        println!("{}", res.status);
        println!("stdout:\n {}\n", String::from_utf8(res.stdout).unwrap());
        println!("stderr:\n {}\n", String::from_utf8(res.stderr).unwrap());
    }
}

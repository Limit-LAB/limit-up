use std::{
    io::{Read, Write},
    process::{Child, Command, Stdio},
};

use super::{ErrorKind, Result};

trait PkgManager {
    fn install(&self, pkgs: &str) -> String;
    fn uninstall(&self, pkgs: &str) -> String;
    fn name(&self) -> &'static str;
}

#[cfg(unix)]
macro_rules! impl_pkg_manager {
    ($class:ident, $name:expr, $install:expr, $uninstall:expr, $update:expr, $flags:expr) => {
        pub struct $class;

        impl PkgManager for $class {
            fn install(&self, pkgs: &str) -> String {
                // {pkgmgr} {update} {flags} && {pkgmgr} {install} {flags} <pkg> && exit
                format!(
                    concat!(
                        $name,
                        " ",
                        $update,
                        " ",
                        $flags,
                        " && ",
                        $name,
                        " ",
                        $install,
                        " ",
                        $flags,
                        " {} && exit\n"
                    ),
                    pkgs
                )
            }

            fn uninstall(&self, pkgs: &str) -> String {
                // {pkgmgr} {uninstall} {flags} <pkg> && exit
                format!(
                    concat!($name, " ", $uninstall, " ", $flags, " {} && exit\n"),
                    pkgs
                )
            }

            fn name(&self) -> &'static str {
                $name
            }
        }
    };
}

#[cfg(unix)]
impl_pkg_manager!(Apt, "apt-get", "install", "remove", "update", "-y");
#[cfg(unix)]
impl_pkg_manager!(Dnf, "dnf", "install", "remove", "update", "-y");
#[cfg(unix)]
impl_pkg_manager!(Pacman, "pacman", "-S", "-Rns", "-Sy", "--noconfirm");
#[cfg(unix)]
impl_pkg_manager!(Zypper, "zypper", "install", "remove", "update", "-y");
#[cfg(unix)]
impl_pkg_manager!(Apk, "apk", "add", "del", "update", ""); // apk does not need

macro_rules! boxed_mgrs {
    ($($mgr:ident),+) => {
        vec![$(Box::new($mgr {})),+]
    };
}

pub struct PackageManager {
    mgr: Box<dyn PkgManager + Send + Sync>,
    proc: Child,
}

impl PackageManager {
    pub fn new() -> Result<PackageManager> {
        Err(ErrorKind::Unsupported.into())
    }

    #[cfg(unix)]
    pub fn new_with_passwd(passwd: impl AsRef<str>) -> Result<PackageManager> {
        use std::{iter::empty, os::fd::AsRawFd};

        use nix::{
            sys::time::{TimeVal, TimeValLike},
            unistd::Uid,
        };

        let root_proc = if Uid::effective().is_root() {
            Command::new("sh")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        } else {
            let mut root_proc = Command::new("su")
                .args(["-c", "sh"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let mut buf = [0; 16];

            if !passwd.as_ref().is_empty() {
                root_proc
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(format!("{}\n", passwd.as_ref()).as_bytes())?;

                root_proc.stderr.as_mut().unwrap().read(&mut buf)?;
                while try_read!(root_proc.stderr.as_mut().unwrap(), buf)? != 0 {}
            }

            loop {
                root_proc.stdin.as_mut().unwrap().write_all(b"whoami\n")?;

                let fdset = select!(
                    root_proc.stdout.as_mut().unwrap(),
                    root_proc.stderr.as_mut().unwrap();
                    &mut TimeVal::seconds(5)
                )?;

                if fdset.contains(root_proc.stdout.as_ref().unwrap().as_raw_fd()) {
                    root_proc.stdout.as_mut().unwrap().read(&mut buf)?;
                    break;
                } else if fdset.contains(root_proc.stderr.as_ref().unwrap().as_raw_fd()) {
                    return Err(ErrorKind::PermissionDenied.into());
                }
            }

            root_proc
        };

        let mgrs: Vec<Box<dyn PkgManager + Send + Sync>> =
            boxed_mgrs![Apt, Dnf, Pacman, Zypper, Apk];

        mgrs.into_iter()
            .find(|mgr| !super::find_command(mgr.name(), empty::<&str>()).is_empty())
            .map(|mgr| PackageManager {
                mgr,
                proc: root_proc,
            })
            .ok_or(ErrorKind::Unsupported.into())
    }

    pub fn install(mut self, pkgs: impl IntoIterator<Item = impl Into<String>>) -> Result<Child> {
        self.proc.stdin.take().unwrap().write_all(
            self.mgr
                .install(
                    pkgs.into_iter()
                        .map(|p| p.into())
                        .collect::<Vec<String>>()
                        .join(" ")
                        .as_str(),
                )
                .as_bytes(),
        )?;

        Ok(self.proc)
    }

    pub fn uninstall(mut self, pkgs: impl IntoIterator<Item = impl Into<String>>) -> Result<Child> {
        self.proc.stdin.take().unwrap().write_all(
            self.mgr
                .uninstall(
                    pkgs.into_iter()
                        .map(|p| p.into())
                        .collect::<Vec<String>>()
                        .join(" ")
                        .as_str(),
                )
                .as_bytes(),
        )?;

        Ok(self.proc)
    }

    pub fn name(&self) -> &'static str {
        self.mgr.name()
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::PackageManager;

    #[cfg(unix)]
    #[test]
    fn pkgmgr_test() {
        let passwd = env::var("PASSWD").unwrap_or_default();

        let res = PackageManager::new_with_passwd(&passwd)
            .map(|mgr| {
                println!("package manager: {}", mgr.name());
                mgr
            })
            .unwrap()
            .install(["cowsay"])
            .unwrap()
            .wait_with_output()
            .unwrap();

        println!("install: {}", res.status);
        println!("stdout:\n{}\n", String::from_utf8(res.stdout).unwrap());
        println!("stderr:\n{}\n", String::from_utf8(res.stderr).unwrap());

        let res = PackageManager::new_with_passwd(&passwd)
            .unwrap()
            .uninstall(["cowsay"])
            .unwrap()
            .wait_with_output()
            .unwrap();

        println!("uninstall: {}", res.status);
        println!("stdout:\n{}\n", String::from_utf8(res.stdout).unwrap());
        println!("stderr:\n{}\n", String::from_utf8(res.stderr).unwrap());
    }
}

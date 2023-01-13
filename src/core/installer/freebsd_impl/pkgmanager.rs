use std::{iter::empty, process::Stdio};
use tokio::{
    io::AsyncWriteExt,
    process::{Child, Command},
};

use r18::tr;

use crate::Result;

trait PkgManager {
    fn install(&self, pkgs: &str) -> String;
    fn uninstall(&self, pkgs: &str) -> String;
    fn name(&self) -> &'static str;
}

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

impl_pkg_manager!(Pkg, "pkg", "install", "autoremove", "update", "-y");

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
        if !nix::unistd::Uid::effective().is_root() {
            Err(tr!("Permission denied, please rerun as Root").to_string())?;
        }

        let root_proc = Command::new("sh")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mgrs: Vec<Box<dyn PkgManager + Send + Sync>> = boxed_mgrs![Pkg];

        mgrs.into_iter()
            .find(|mgr| !super::find_command(mgr.name(), empty::<&str>()).is_empty())
            .map(|mgr| PackageManager {
                mgr,
                proc: root_proc,
            })
            .ok_or(tr!("Unsupported platform").into())
    }

    pub async fn install(
        mut self,
        pkgs: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Child> {
        self.proc
            .stdin
            .take()
            .unwrap()
            .write_all(
                self.mgr
                    .install(
                        pkgs.into_iter()
                            .map(|p| p.into())
                            .collect::<Vec<String>>()
                            .join(" ")
                            .as_str(),
                    )
                    .as_bytes(),
            )
            .await?;

        Ok(self.proc)
    }

    pub async fn uninstall(
        mut self,
        pkgs: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Child> {
        self.proc
            .stdin
            .take()
            .unwrap()
            .write_all(
                self.mgr
                    .uninstall(
                        pkgs.into_iter()
                            .map(|p| p.into())
                            .collect::<Vec<String>>()
                            .join(" ")
                            .as_str(),
                    )
                    .as_bytes(),
            )
            .await?;

        Ok(self.proc)
    }

    pub fn name(&self) -> &'static str {
        self.mgr.name()
    }
}

#[cfg(test)]
mod tests {
    use super::PackageManager;
    use crate::core::RT;

    #[test]
    fn pkgmgr_test() {
        let res = RT.block_on(async {
            PackageManager::new()
            .map(|mgr| {
                println!("package manager: {}", mgr.name());
                mgr
            })
            .unwrap()
            .install(["cowsay"])
            .await
            .unwrap()
            .wait_with_output()
            .await
            .unwrap()
        });
        
        println!("install: {}", res.status);
        println!("stdout:\n{}\n", String::from_utf8(res.stdout).unwrap());
        println!("stderr:\n{}\n", String::from_utf8(res.stderr).unwrap());

        let res = RT.block_on(async {
            PackageManager::new()
            .unwrap()
            .uninstall(["cowsay"])
            .await
            .unwrap()
            .wait_with_output()
            .await
            .unwrap()
        });

        println!("uninstall: {}", res.status);
        println!("stdout:\n{}\n", String::from_utf8(res.stdout).unwrap());
        println!("stderr:\n{}\n", String::from_utf8(res.stderr).unwrap());
    }
}

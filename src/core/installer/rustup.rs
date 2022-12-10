use std::{
    env,
    process::{Child, Command, Stdio},
};

use super::Result;

pub struct Rustup;

#[cfg(target_family = "windows")]
const DELIMITER: char = ';';

#[cfg(target_family = "unix")]
const DELIMITER: char = ':';

impl Rustup {
    #[cfg(target_family = "windows")]
    pub fn install() -> Result<Self> {
        Err(Error::NotSupported)
    }

    #[cfg(target_family = "unix")]
    pub fn install() -> Result<Child> {
        use std::io::Write;

        let mut proc = Command::new("/usr/bin/bash")
            .args(["-s", "--", "-y", "--default-toolchain", "none"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let resp = reqwest::blocking::get("https://sh.rustup.rs")?.bytes()?;

        proc.stdin.take().unwrap().write_all(&resp)?;

        Ok(proc)
    }

    pub fn uninstall() -> Result<Child> {
        let mut path = env::var("PATH")
            .map(|mut p| {
                p.push(DELIMITER);
                p
            })
            .unwrap_or_default();

        path.push_str("~/.cargo/bin");

        let proc = Command::new("rustup")
            .env("PATH", path)
            .args(["self", "uninstall", "-y"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(proc)
    }
}

#[cfg(test)]
mod tests {
    use super::Rustup;

    #[test]
    fn rustup_test() {
        let res = Rustup::install().unwrap().wait_with_output().unwrap();

        println!("install: {}", res.status);
        println!("stdout:\n{}\n", String::from_utf8(res.stdout).unwrap());
        println!("stderr:\n{}\n", String::from_utf8(res.stderr).unwrap());

        let res = Rustup::uninstall().unwrap().wait_with_output().unwrap();

        println!("uninstall: {}", res.status);
        println!("stdout:\n{}\n", String::from_utf8(res.stdout).unwrap());
        println!("stderr:\n{}\n", String::from_utf8(res.stderr).unwrap());
    }
}
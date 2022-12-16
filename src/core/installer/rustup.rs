use std::{
    env,
    process::{Child, Command, Stdio},
};

use super::{Error, ErrorKind, Result};

pub struct Rustup;

#[cfg(windows)]
const DELIMITER: char = ';';

#[cfg(unix)]
const DELIMITER: char = ':';

impl Rustup {
    #[cfg(windows)]
    pub fn install() -> Result<Child> {
        Err(ErrorKind::Unsupported.into())
    }

    #[cfg(unix)]
    pub fn install() -> Result<Child> {
        let mut curl = Command::new("curl")
            .args([
                "--proto",
                "=https",
                "-tlsv1.2",
                "-sSf",
                "https://sh.rustup.rs",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let proc = Command::new("sh")
            .args(["-s", "--", "-y", "--default-toolchain", "nightly"])
            .stdin(curl.stdout.take().unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let curl_res = curl.wait_with_output()?;

        if !curl_res.status.success() {
            return Err(Error::new(
                ErrorKind::Other,
                String::from_utf8(curl_res.stderr.into()).unwrap(),
            ));
        }

        Ok(proc)
    }

    pub fn uninstall() -> Result<Child> {
        let mut path = env::var("PATH")
            .map(|mut p| {
                p.push(DELIMITER);
                p
            })
            .unwrap_or_default();

        path.push_str(&format!(
            "{}/.cargo/bin",
            env::var("HOME").map_err(|_| Error::new(ErrorKind::NotFound, "Invalid home path"))?
        ));

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
    use std::{env, iter::empty};

    use crate::core::installer::{find_command, PackageManager};

    use super::Rustup;

    #[cfg(unix)]
    #[test]
    fn rustup_test() {
        if find_command("curl", empty::<&str>()).is_empty() {
            let res = PackageManager::new_with_passwd(env::var("PASSWD").unwrap_or_default())
                .unwrap()
                .install(["curl"])
                .unwrap()
                .wait_with_output()
                .unwrap();

            println!("install curl: {}", res.status);
            println!("stdout:\n{}\n", String::from_utf8(res.stdout).unwrap());
            println!("stderr:\n{}\n", String::from_utf8(res.stderr).unwrap());

            if !res.status.success() {
                panic!("install curl failed");
            }
        }

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

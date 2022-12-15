use std::process::{Child, Command, Stdio};

use super::Result;

pub struct Cargo {
    path: String,
}

impl Cargo {
    pub fn new(path: String) -> Self {
        Self { path: path }
    }

    pub fn install(self, install_root: String) -> Result<Child> {
        Command::new(self.path)
            .args([
                "install",
                "--root",
                install_root.as_ref(),
                "--git",
                "https://github.com/Limit-IM/limit-server",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

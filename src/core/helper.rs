use std::collections::HashMap;

use once_cell::sync::Lazy;

static PKGMGR_HELP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    [(
        "dnf",
        "Please confirm the network settings and RHEL Subscription is enabled. \
if problem persists please contact us.",
    )]
    .into_iter()
    .collect()
});

pub enum Help {
    Authorization,
    Dependencies(
        &'static str, // package manager name
    ),
    InitRust,
    InstallServer,
}

const NETWORK_HELP: &'static str = "Please confirm the network settings and try again. \
If the problem persists please contact us";

impl Help {
    pub fn info(&self) -> &'static str {
        match *self {
            Help::Dependencies(name) => PKGMGR_HELP.get(name).unwrap_or(&NETWORK_HELP),
            Help::Authorization => "Please confirm the password and try again",
            Help::InitRust => NETWORK_HELP,
            Help::InstallServer => NETWORK_HELP,
        }
    }
}

impl std::fmt::Display for Help {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "help: {}", self.info())
    }
}

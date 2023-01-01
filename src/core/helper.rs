use std::collections::HashMap;

use once_cell::sync::Lazy;
use r18::tr;

static PKGMGR_HELP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    [(
        "dnf",
        tr!("Please confirm the network settings and RHEL Subscription is enabled. \
if problem persists please contact us."),
    )]
    .into_iter()
    .collect()
});

pub enum Help {
    PackageManager(
        &'static str, // package manager name
    ),
    InitRust,
    InstallServer,
}

static NETWORK_HELP: Lazy<&'static str> = Lazy::new(|| tr!("Please confirm the network settings and try again. \
If the problem persists please contact us."));

impl Help {
    pub fn info(&self) -> &'static str {
        match *self {
            Help::PackageManager(name) => PKGMGR_HELP.get(name).unwrap_or(&*NETWORK_HELP),
            Help::InitRust => *NETWORK_HELP,
            Help::InstallServer => *NETWORK_HELP,
        }
    }
}

impl std::fmt::Display for Help {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", tr!("help: {}", self.info()))
    }
}

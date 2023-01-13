use once_cell::sync::Lazy;
use r18::tr;

pub enum Help {
    Git,
    Network,
}

static CONTACT_US: Lazy<&'static str> =
    Lazy::new(|| tr!("if the problem persists please contact us."));

impl Help {
    pub fn info(&self) -> String {
        match *self {
            Help::Network => tr!(
                "Please confirm the network settings and try again, {}",
                &*CONTACT_US
            ),
            Help::Git => tr!(
                "Check your network settings or delete the repository and try again, {}",
                &*CONTACT_US
            ),
        }
        .to_string()
    }
}

impl std::fmt::Display for Help {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", tr!("help: {}", self.info()))
    }
}

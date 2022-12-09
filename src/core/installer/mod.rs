#[macro_export]
macro_rules! try_read {
    ($pipe:expr, $buf:expr) => {{
        let output = $pipe.as_mut().unwrap();

        let mut fdset = FdSet::new();
        fdset.insert(output.as_raw_fd());
        select(
            output.as_raw_fd() + 1,
            &mut fdset,
            None,
            None,
            &mut TimeVal::milliseconds(0),
        )
        .map(|ret| ret as usize)
        .map_err(|e| e.into())
        .and_then(|ret| match ret {
            0 => Ok(0),
            1 => output.read(&mut $buf),
            _ => unreachable!(),
        })
    }};
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    NetworkError(reqwest::Error),
    NotSupported,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::NetworkError(e)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::IoError(ref e) => write!(f, "{}", e),
            Error::NetworkError(ref e) => write!(f, "{}", e),
            Error::NotSupported => write!(f, "Unsupported platform"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

mod_use::mod_use!(pkgmanager, rustup);

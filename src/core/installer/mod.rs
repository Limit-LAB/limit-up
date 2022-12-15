use std::{
    env,
    path::{Path, PathBuf},
};

#[macro_export]
macro_rules! select {
    ($($pipe:expr),+; $timeout:expr) => {{
        use std::os::fd::AsRawFd;
        use nix::sys::select::{select, FdSet};

        let mut fdset = FdSet::new();
        $(
            fdset.insert($pipe.as_raw_fd());
        )+

        select(
            fdset.highest().unwrap() + 1,
            &mut fdset,
            None,
            None,
            $timeout,
        )
        .map(|_| fdset)
    }}
}

#[macro_export]
macro_rules! try_read {
    ($pipe:expr, $buf:expr) => {{
        match select!($pipe; &mut TimeVal::milliseconds(0)) {
            Ok(fdset) => {
                let output = $pipe;
                match fdset.contains(output.as_raw_fd()) {
                    true => output.read(&mut $buf),
                    false => Ok(0)
                }
            },
            Err(e) => Err($crate::core::installer::Error::from(e)),
        }
    }};
}

#[inline]
pub fn find_command(
    program: impl AsRef<Path>,
    other: impl IntoIterator<Item = impl Into<PathBuf>>,
) -> Vec<PathBuf> {
    env::var_os("PATH")
        .map(|paths| {
            env::split_paths(&paths)
                .chain(other.into_iter().map(|p| p.into()))
                .filter_map(|path| {
                    #[cfg(target_family = "windows")]
                    let full_path = path.join(format!("{}.exe", program.as_ref().display()));

                    #[cfg(target_family = "unix")]
                    let full_path = path.join(program.as_ref());

                    full_path.is_file().then_some(full_path)
                })
                .collect()
        })
        .unwrap_or_default()
}

pub type Error = std::io::Error;
pub type ErrorKind = std::io::ErrorKind;
pub type Result<T> = std::io::Result<T>;

mod_use::mod_use!(pkgmanager, rustup, cargo);

#[cfg(test)]
mod tests {
    use std::env;

    use super::find_command;

    #[test]
    fn test_find_command() {
        let paths = find_command(
            "cargo",
            env::var("HOME")
                .map(|s| vec![format!("{}/.cargo/bin", s)])
                .unwrap_or_default(),
        );

        println!("{:#?}", paths);
    }
}

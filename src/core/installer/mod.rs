use std::{
    env,
    path::{Path, PathBuf},
};

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

mod_use::mod_use!(pkgmanager, rustup);

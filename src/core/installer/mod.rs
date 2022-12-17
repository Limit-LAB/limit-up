use std::{
    env,
    path::{Path, PathBuf},
};

#[cfg(unix)]
#[macro_export]
macro_rules! as_raw {
    ($pipe:expr) => {{
        use std::os::fd::AsRawFd;
        $pipe.as_raw_fd()
    }};
}

#[cfg(windows)]
#[macro_export]
macro_rules! as_raw {
    ($pipe:expr) => {{
        use std::{mem, os::windows::prelude::AsRawHandle};

        unsafe { mem::transmute($pipe.as_raw_handle()) }
    }};
}

#[cfg(unix)]
#[macro_export]
macro_rules! select {
    ($($pipe:expr),+; $timeout:expr) => {{
        use nix::sys::select::{select, FdSet};

        let mut fdset = FdSet::new();
        $(
            fdset.insert(as_raw!($pipe));
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

#[cfg(windows)]
#[macro_export]
macro_rules! select {
    ($($pipe:expr),+; $timeout:expr) => {{
        use windows::Win32::{
            Foundation::WAIT_OBJECT_0, System::Threading::WaitForMultipleObjects,
        };

        #[allow(unused_unsafe)]
        unsafe {
            let mut once = true;
            let mut index = 0;
            let mut ret = Vec::new();
            let handles = vec![$(as_raw!($pipe),)+];
            loop {
                let res = WaitForMultipleObjects(
                    &handles[index..],
                    false,
                    if once {
                        once = false;
                        $timeout
                    } else {
                        0
                    },
                );

                // valid index
                if res.0 < WAIT_OBJECT_0.0 + handles.len() as u32 {
                    ret.push(handles[res.0 as usize]);

                    // is last item
                    if res.0 == handles.len() as u32 - 1 {
                        break;
                    }

                    index = res.0 as usize + 1;
                } else {
                    break;
                }
            }

            ret
        }
    }}
}

#[cfg(unix)]
#[macro_export]
macro_rules! try_read {
    ($pipe:expr, $buf:expr) => {{
        match select!($pipe; &mut TimeVal::milliseconds(0)) {
            Ok(fdset) => {
                match fdset.contains(as_raw!($pipe)) {
                    true => $pipe.read(&mut $buf),
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
                    #[cfg(windows)]
                    let full_path = path.join(format!("{}.exe", program.as_ref().display()));

                    #[cfg(unix)]
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

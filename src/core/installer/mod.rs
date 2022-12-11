use std::process::Command;

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
pub fn command_exists(program: impl AsRef<str>) -> bool {
    Command::new(program.as_ref()).output().is_ok()
}

pub type Error = std::io::Error;
pub type ErrorKind = std::io::ErrorKind;
pub type Result<T> = std::io::Result<T>;

mod_use::mod_use!(pkgmanager, rustup);

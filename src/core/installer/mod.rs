use std::{
    env,
    path::{Path, PathBuf},
};

/// Return paths of the program
#[allow(dead_code)]
pub fn find_command(
    program: impl AsRef<Path>,
    other: impl IntoIterator<Item = impl Into<PathBuf>>,    // extra path
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

#[derive(Default)]
pub struct InstallConfig {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    pub install_root: String,
}

#[cfg(target_os = "linux")]
mod_use::mod_use!(linux_impl);

#[cfg(target_os = "freebsd")]
mod_use::mod_use!(freebsd_impl);

#[cfg(target_os = "windows")]
mod_use::mod_use!(windows_impl);

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

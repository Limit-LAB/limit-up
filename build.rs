use build_target::{target_os, target_env, target_arch};

fn main() {
    println!("cargo:rustc-env=TARGET_OS={}", target_os().unwrap());
    println!("cargo:rustc-env=TARGET_ENV={}", target_env().unwrap());
    println!("cargo:rustc-env=TARGET_ARCH={}", target_arch().unwrap());
}

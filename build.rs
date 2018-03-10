fn main() {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-search=native=/usr/local/opt/readline/lib");
}

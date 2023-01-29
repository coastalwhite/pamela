fn main() {
    pkg_config::probe_library("pam").expect("Failed to find libpam.so");
}
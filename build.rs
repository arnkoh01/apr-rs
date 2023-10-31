extern crate bindgen;
extern crate pkg_config;

fn create_bindings(out_path: &std::path::Path) {
    let pc_apr = pkg_config::Config::new()
        .probe("apr-1")
        .unwrap_or_else(|e| panic!("Failed to find apr library: {}", e));

    let apr_path = pc_apr
        .include_paths
        .iter()
        .find(|x| x.join("apr.h").exists())
        .expect("Failed to find apr.h");

    // Generate bindings using bindgen
    let bindings = bindgen::Builder::default()
        .header(apr_path.join("apr.h").to_str().unwrap())
        .header(apr_path.join("apr_allocator.h").to_str().unwrap())
        .header(apr_path.join("apr_general.h").to_str().unwrap())
        .header(apr_path.join("apr_errno.h").to_str().unwrap())
        .header(apr_path.join("apr_pools.h").to_str().unwrap())
        .header(apr_path.join("apr_version.h").to_str().unwrap())
        .allowlist_file(".*/apr.h")
        .allowlist_file(".*/apr_general.h")
        .allowlist_file(".*/apr_allocator.h")
        .allowlist_file(".*/apr_version.h")
        .allowlist_file(".*/apr_errno.h")
        .allowlist_file(".*/apr_pools.h")
        .clang_args(
            pc_apr
                .include_paths
                .iter()
                .map(|path| format!("-I{}", path.display())),
        )
        .generate()
        .expect("Failed to generate bindings");

    bindings
        .write_to_file(out_path.join("generated.rs"))
        .expect("Failed to write bindings");
}

fn main() {
    system_deps::Config::new().probe().unwrap();

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    create_bindings(out_path.as_path());
}
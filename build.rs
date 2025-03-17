use vcpkg::Config;

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        if std::env::var_os("CARGO_CFG_TARGET_ENV").is_some_and(|env| env != "msvc") {
            println!("cargo::error=Non-MSVC environment is not supported!");
            return
        }

        let Some(arch) = std::env::var_os("CARGO_CFG_TARGET_ARCH") else {
            println!("cargo::error=`CARGO_CFG_TARGET_ARCH` not found!");
            return
        };

        // Safety:
        // `build.rs` is single-threaded.
        // This is required to explicitly allow `find_package(...)` to link against DLLs.
        unsafe { std::env::set_var("VCPKGRS_DYNAMIC", "1") };

        match Config::new()
            .target_triplet(if arch == "x86" {
                "x86-windows"
            } else if arch == "x86_64" {
                "x64-windows"
            } else if arch == "aarch64" {
                "arm64-windows"
            } else {
                println!("cargo::error=Unsupported architecture: `{arch:?}`!");
                return
            })
            .find_package("mimalloc")
        {
            Ok(..) => {
                // Include the `mi_version` symbol to force linking against `mimalloc.dll`.
                println!("cargo::rustc-link-arg=/include:mi_version")
            }
            Err(e) => println!("cargo::error={e}"),
        }
    }
}

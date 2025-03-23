#![feature(let_chains)]

use std::{
    env::{var, var_os},
    fs::File,
    io::{Result as IoResult, Write},
    path::PathBuf,
};

fn main() -> IoResult<()> {
    let Some(path) = PathBuf::from(var_os("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .map(|p| p.join("centripetal-assets"))
    else {
        return Ok(())
    };

    let asset_directory = if let Ok("debug") = var("PROFILE").as_deref() &&
        match var("CE_NO_LOCAL_ASSETS")
            .map(|mut s| {
                s.make_ascii_lowercase();
                s
            })
            .as_deref()
        {
            Ok("1" | "true" | "on" | "yes") => false,
            _ => true,
        } &&
        path.exists() &&
        let Some(path) = path.to_str()
    {
        println!("cargo::rerun-if-changed=../centripetal-assets");
        format!(
            "pub const ASSET_DIRECTORY: &'static str = \"{}\";",
            path.replace('\\', "\\\\")
        )
    } else {
        "pub const ASSET_DIRECTORY: &'static str = \"assets\";".into()
    };

    File::create(format!("{}/asset_directory.rs", var("OUT_DIR").unwrap()))?.write_all(asset_directory.as_bytes())
}

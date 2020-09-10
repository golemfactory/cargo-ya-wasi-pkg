use cargo_metadata::MetadataCommand;

use std::error::Error;
use std::fs;
use std::process::Command;
use structopt::StructOpt;

const PKG_EXTENSION: &str = "ywasi";

mod manifest;
mod repo;

use manifest::*;

#[derive(StructOpt)]
struct Options {
    #[structopt(long)]
    publish: bool,
    #[structopt(long)]
    debug: bool,
    #[structopt(long)]
    show_manifest: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Options::from_iter(std::env::args_os().skip(1));

    #[derive(Debug)]
    struct WasmPackage {
        name: String,
        meta: Manifest,
    }

    let metadata = MetadataCommand::new().exec()?;
    let wasi_packages = metadata.packages.into_iter().filter_map(|mut p| {
        if p.source.is_some() {
            return None;
        }
        let meta = p.metadata.as_object_mut()?.remove("ya-wasi-pkg")?;
        let entry_points = p
            .targets
            .into_iter()
            .filter_map(|target| {
                if target.kind.iter().all(|n| n != "bin") {
                    return None;
                }
                let wasm_path = format!("{}.wasm", target.name);
                Some(EntryPoint {
                    id: target.name,
                    wasm_path,
                    args_prefix: Default::default(),
                })
            })
            .collect();
        let name = p.name;

        let mut meta: Manifest = serde_json::from_value(meta).unwrap();
        if meta.entry_points.is_empty() {
            meta.entry_points = entry_points;
        }
        if meta.id.is_none() {
            meta.id = Some(format!("network.golem/ya-pkg/{}", &name))
        }
        if meta.name.is_none() {
            meta.name = Some(name.clone());
        }
        Some(WasmPackage { name, meta })
    });
    let bin_dir = metadata
        .target_directory
        .join("wasm32-wasi")
        .join("release");
    let out_dir = metadata.target_directory.join("ya-pkg");
    std::fs::create_dir_all(&out_dir)?;
    for package in wasi_packages {
        let _ = Command::new("cargo")
            .args(&[
                "build",
                "--release",
                "--target",
                "wasm32-wasi",
                "-p",
                package.name.as_ref(),
                "--bins",
            ])
            .status()?;

        if args.show_manifest {
            println!("{}", serde_json::to_string_pretty(&package.meta).unwrap());
        }

        let output = {
            let out_file_name = out_dir.join(&package.name).with_extension(PKG_EXTENSION);
            let out_file = std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&out_file_name)?;
            let mut pkg_file: zip::ZipWriter<_> = zip::ZipWriter::new(out_file);
            pkg_file.start_file(
                "manifest.json",
                zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )?;
            serde_json::to_writer_pretty(&mut pkg_file, &package.meta)?;
            for bin in package.meta.entry_points {
                let wasm_file = bin_dir.join(&bin.wasm_path);
                pkg_file.start_file(&bin.wasm_path, Default::default())?;
                std::io::copy(
                    &mut fs::OpenOptions::new().read(true).open(wasm_file)?,
                    &mut pkg_file,
                )?;
            }
            pkg_file.finish()?;
            eprintln!("generated package: {}", out_file_name.display());
            out_file_name
        };
        if args.publish {
            eprintln!("pushing image to repo");
            let url = repo::push_file(output)?;
            eprintln!("published package {}", url);
        }
    }

    Ok(())
}

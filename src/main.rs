
use cargo_metadata::MetadataCommand;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>>{
    for p in MetadataCommand::new().exec()?.packages {
        if p.source.is_none() {
            eprintln!("p={}", p.name);
            eprintln!("m={}", p.manifest_path.display());
            eprintln!("mx={:?}", p.metadata);
        }
    }
    Ok(())
}

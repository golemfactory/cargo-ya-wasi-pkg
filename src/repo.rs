
use sha3::{digest::Digest, Sha3_224};

use std::path::Path;
use std::{fs, io};
use trust_dns_resolver::config::*;
use trust_dns_resolver::proto::rr::{RData, RecordType};
use trust_dns_resolver::Resolver;

const DEV_SERVICE_NAME: &str = "_girepo._tcp.dev.golem.network";

fn find_repo() -> anyhow::Result<String> {
    let resolver = Resolver::new(ResolverConfig::google(), ResolverOpts::default())?;
    for record in resolver.lookup(DEV_SERVICE_NAME, RecordType::SRV)? {
        if let RData::SRV(ref srv) = record {
            let port = srv.port();
            let target = srv.target().to_string();
            let base_url = format!("http://{}:{}", target, port);
            let q = reqwest::blocking::get(&format!("{}/status", &base_url));
            if let Ok(resp) = q {
                if resp.status().is_success() {
                    return Ok(base_url);
                }
            }
        }
    }
    anyhow::bail!("repo not found")
}

pub fn push_file(file_path: impl AsRef<Path>) -> anyhow::Result<String> {
    let file_path = file_path.as_ref();
    let file_name_bytes: &Path = file_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("invalid file name"))?
        .as_ref();
    let file_name = file_name_bytes.display().to_string();
    let pos = file_name.rfind(".").unwrap();

    let base_name = &file_name[..pos];
    let file_ext = &file_name[(pos + 1)..];

    let base_url = find_repo()?;
    let mut hasher = Sha3_224::default();
    io::copy(&mut fs::File::open(file_path)?, &mut hasher)?;
    let hash = format!("{:x}", hasher.finalize());
    let image_name = format!("{}-{}.{}", base_name, &hash[(hash.len() - 20)..], file_ext);
    let download_url = format!("{}/{}", base_url, image_name);
    {
        let client = reqwest::blocking::Client::builder().build()?;

        let upload_url = format!("{}/upload/{}", base_url, image_name);
        client
            .post(&upload_url)
            .body(fs::File::open(file_path)?)
            .send()?;
    }
    {
        let client = reqwest::blocking::Client::builder().build()?;
        client
            .post(&format!("{}/upload/image.{}.link", base_url, hash))
            .body(download_url.as_bytes().to_vec())
            .send()?;
    }

    Ok(format!("hash:sha3:{}:{}", hash, download_url))
}

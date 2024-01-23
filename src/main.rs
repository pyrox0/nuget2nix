#![warn(clippy::pedantic)]
use anyhow::anyhow;
use camino::Utf8PathBuf;
use nuget::{download_url, version_exists};
use pico_args::Arguments;
use std::sync::Arc;
use url::Url;

use crate::nuget::NuGet;
use crate::nuget::PackageData;

mod nix_hash;
mod nuget;

struct Res {
    pname: String,
    version: String,
    url: Url,
    sha256: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = Arguments::from_env();
    let dir: Utf8PathBuf = args.value_from_str("--directory")?;

    let nuget = Arc::new(NuGet::new(dir)?);

    let mut futures = Vec::new();
    for pkg in nuget.packages.clone() {
        let nuget = nuget.clone();
        let fut = tokio::spawn(async move { get_fetch_nuget_args(&pkg, nuget).await });
        futures.push(fut);
    }

    println!("{{fetchNuGet}}: [");

    for fut in futures {
        let res = fut.await??;

        println!(
            "  (fetchNuGet {{ pname = \"{}\"; version = \"{}\"; url = \"{}\"; sha256 = \"{}\"; }})",
            res.pname, res.version, res.url, res.sha256
        );
    }
    println!("]");

    Ok(())
}

async fn get_fetch_nuget_args(pkg: &PackageData, nuget: Arc<NuGet>) -> anyhow::Result<Res> {
    let package_base_address = nuget.get_package_base_address(&pkg).await?;

    let package_id = pkg.id.to_string();
    let package_versions = nuget
        .get_package_versions(&pkg, &package_base_address)
        .await?;

    if version_exists(&pkg, &package_versions) {
        let sha256 = nix_hash::hash(&pkg.nupkg_path)?;

        let url = download_url(&package_base_address, &package_id, &pkg.version)?;

        Ok(Res {
            pname: pkg.id.clone(),
            version: pkg.version.clone(),
            url,
            sha256,
        })
    } else {
        return Err(anyhow!(
            "couldn't find repo with {} v{}",
            pkg.id,
            pkg.version
        ));
    }
}

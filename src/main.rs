#![warn(clippy::pedantic)]
use nuget::download_url;
use std::env;
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

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let dir = &args[1];

    let nuget = Arc::new(NuGet::new(dir.into())?);

    let mut futures = Vec::new();
    for pkg in nuget.packages.clone() {
        let nuget = nuget.clone();
        let fut = get_fetch_nuget_args(&pkg, nuget);
        futures.push(fut);
    }

    println!("{{fetchNuGet}}: [");

    for fut in futures {
        let res = fut?;

        println!(
            "  (fetchNuGet {{ pname = \"{}\"; version = \"{}\"; url = \"{}\"; sha256 = \"{}\"; }})",
            res.pname, res.version, res.url, res.sha256
        );
    }
    println!("]");

    Ok(())
}

fn get_fetch_nuget_args(pkg: &PackageData, nuget: Arc<NuGet>) -> anyhow::Result<Res> {
    let package_base_address = nuget.get_package_base_address(&pkg)?;
    let package_id = pkg.id.to_string();
    let url = download_url(&package_base_address, &package_id, &pkg.version)?;
    let sha256 = nix_hash::hash(&pkg.nupkg_path)?;

    Ok(Res {
        pname: pkg.id.clone(),
        version: pkg.version.clone(),
        url,
        sha256,
    })
}

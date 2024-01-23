#![warn(clippy::pedantic)]
use std::env;

use crate::nuget::NuGet;

mod nix_hash;
mod nuget;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let package_path = &args[1];

    let nuget = NuGet::new(package_path.into())?;

    println!("{{fetchNuGet}}: [");

    for pkg in nuget.packages.clone() {
        let pname = &pkg.id;
        let version = &pkg.version;
        let url = nuget.get_download_url(&pkg)?;
        let sha256 = nix_hash::hash(&pkg.nupkg_path)?;

        println!(
            "  (fetchNuGet {{ pname = \"{}\"; version = \"{}\"; url = \"{}\"; sha256 = \"{}\"; }})",
            pname, version, url, sha256
        );
    }

    println!("]");

    Ok(())
}

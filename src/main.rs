#![warn(clippy::pedantic)]
use camino::Utf8PathBuf;
use std::env;

use crate::nuget::NuGet;

mod nix_hash;
mod nuget;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let package_path = &args[1];

    let exclude_file: Option<Utf8PathBuf> = if args.len() > 2 {
        Some((&args[2]).into())
    } else {
        None
    };

    let nuget = NuGet::new(package_path.into(), exclude_file)?;

    println!("{{fetchNuGet}}: [");

    for pkg in nuget.packages.clone() {
        let pname = &pkg.id;
        let version = &pkg.version;
        let sha256 = nix_hash::hash(&pkg.nupkg_path)?;

        if pkg.source.to_string() != "https://api.nuget.org/v3/index.json" {
            let url = nuget.get_download_url(&pkg)?;

            println!(
                "  (fetchNuGet {{ pname = \"{}\"; version = \"{}\"; sha256 = \"{}\"; url = \"{}\"; }})",
                pname, version, sha256, url
            );
        } else {
            println!(
                "  (fetchNuGet {{ pname = \"{}\"; version = \"{}\"; sha256 = \"{}\"; }})",
                pname, version, sha256
            );
        }
    }

    println!("]");

    Ok(())
}

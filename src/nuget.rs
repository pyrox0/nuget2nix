use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, BufRead},
    path::Path,
};

use anyhow::Error;
use camino::Utf8PathBuf;
use glob::glob;
use quick_cache::sync::Cache;
use reqwest::blocking::Client;
use serde::Deserialize;
use url::Url;

pub struct NuGet {
    client: Client,
    package_base_address_cache: Cache<String, Url>,
    pub packages: Vec<Package>,
}

impl NuGet {
    pub fn new(
        package_dir: Utf8PathBuf,
        exclude_file: Option<Utf8PathBuf>,
    ) -> anyhow::Result<NuGet> {
        let packages = read_package_dir(package_dir, exclude_file)?;

        return Ok(NuGet {
            client: Client::new(),
            package_base_address_cache: Cache::new(10),
            packages,
        });
    }

    pub fn get_download_url(&self, pkg: &Package) -> anyhow::Result<Url> {
        let package_base_address = self.get_package_base_address(&pkg)?;
        let package_id = &pkg.id;
        let version = &pkg.version;

        return Ok(package_base_address.join(&format!(
            "{package_id}/{version}/{package_id}.{version}.nupkg"
        ))?);
    }

    fn get_package_base_address(&self, pkg: &Package) -> anyhow::Result<Url> {
        let source = &pkg.source;

        let package_base_address =
            self.package_base_address_cache
                .get_or_insert_with(&source.to_string(), || {
                    let index: ServiceIndex = self.client.get(source.clone()).send()?.json()?;

                    let mut package_base_address = index
                        .resources
                        .into_iter()
                        .find(|r| r.typ == "PackageBaseAddress/3.0.0")
                        .unwrap()
                        .url;

                    package_base_address
                        .path_segments_mut()
                        .map_err(|_| Error::msg("cannot-be-a-base"))?
                        .pop_if_empty()
                        .push("");

                    Ok::<Url, Error>(package_base_address)
                })?;

        Ok(package_base_address)
    }
}

fn read_package_dir(
    package_dir: Utf8PathBuf,
    exclude_file: Option<Utf8PathBuf>,
) -> anyhow::Result<Vec<Package>> {
    let mut packages = Vec::new();

    let excluded_packages: HashSet<String> = match exclude_file {
        Some(it) => HashSet::from_iter(read_lines(it)?.flatten()),
        None => HashSet::new(),
    };

    for mut path in utf8_glob(package_dir.join("**/*.nuspec").as_str()) {
        let nuspec: Nuspec = quick_xml::de::from_str(&fs::read_to_string(&path)?)?;

        assert!(path.pop());

        let nupkg_path: Utf8PathBuf = utf8_glob(path.join("*.nupkg").as_str()).next().unwrap();

        // Skip packages that have been excluded
        if excluded_packages.contains(&nupkg_path.file_name().unwrap().to_owned()) {
            continue;
        }

        let nupkg_metadata_path = glob(path.join(".nupkg.metadata").as_str())?
            .next()
            .unwrap()?;

        let nupkg_metadata: NupkgMetadata =
            serde_json::from_str(&fs::read_to_string(&nupkg_metadata_path)?)?;

        // Skip non-url sources
        let source = match Url::parse(&nupkg_metadata.source) {
            Ok(it) => it,
            Err(_) => continue,
        };

        packages.push(Package {
            id: nuspec.metadata.id,
            version: nuspec.metadata.version,
            source: source,
            nupkg_path,
        });
    }

    Ok(packages)
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn utf8_glob(pattern: &str) -> impl Iterator<Item = Utf8PathBuf> {
    glob(pattern)
        .expect("Failed to read glob pattern")
        .map(|path| Utf8PathBuf::from_path_buf(path.unwrap()).unwrap())
}

#[derive(Deserialize)]
struct ServiceIndex {
    resources: Vec<Resource>,
}

#[derive(Deserialize)]
struct Resource {
    #[serde(rename = "@id")]
    url: Url,
    #[serde(rename = "@type")]
    typ: String,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub id: String,
    pub version: String,
    pub source: Url,
    pub nupkg_path: Utf8PathBuf,
}

#[derive(Deserialize)]
struct NupkgMetadata {
    source: String,
}

#[derive(Deserialize)]
struct Nuspec {
    metadata: NuspecMetadata,
}

#[derive(Deserialize)]
struct NuspecMetadata {
    id: String,
    version: String,
}

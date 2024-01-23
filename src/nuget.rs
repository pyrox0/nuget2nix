use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Error;
use glob::glob;
use quick_cache::sync::Cache;
use reqwest::blocking::Client;
use serde::Deserialize;
use url::Url;

pub struct NuGet {
    client: Arc<Client>,
    cache: Arc<MyCache>,
    pub packages: Vec<PackageData>,
}

impl NuGet {
    pub fn new(package_dir: PathBuf) -> anyhow::Result<NuGet> {
        let packages = read_package_dir(package_dir)?;

        return Ok(NuGet {
            client: Arc::new(Client::new()),
            cache: Arc::new(MyCache::new()),
            packages,
        });
    }

    pub fn get_package_base_address(&self, pkg: &PackageData) -> anyhow::Result<Url> {
        let source = &pkg.source;

        let package_base_address =
            self.cache
                .package_base_address
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

pub fn download_url(
    package_base_address: &Url,
    package_id: &String,
    version: &String,
) -> anyhow::Result<Url> {
    return Ok(package_base_address.join(&format!(
        "{package_id}/{version}/{package_id}.{version}.nupkg"
    ))?);
}

fn read_package_dir(package_dir: PathBuf) -> anyhow::Result<Vec<PackageData>> {
    let mut packages = Vec::new();

    for mut path in glob(package_dir.join("**/*.nuspec").to_str().unwrap())?.map(Result::unwrap) {
        let nuspec: Nuspec = quick_xml::de::from_str(&fs::read_to_string(&path)?)?;

        assert!(path.pop());

        let nupkg_path = glob(path.join("*.nupkg").to_str().unwrap())?
            .next()
            .unwrap()?;

        let nupkg_metadata_path = glob(path.join(".nupkg.metadata").to_str().unwrap())?
            .next()
            .unwrap()?;

        let nupkg_metadata: NupkgMetadata =
            serde_json::from_str(&fs::read_to_string(&nupkg_metadata_path)?)?;

        packages.push(PackageData::new(nuspec, nupkg_path, nupkg_metadata));
    }

    Ok(packages)
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
pub struct PackageData {
    pub id: String,
    pub version: String,
    pub source: Url,
    pub nupkg_path: PathBuf,
}

impl PackageData {
    fn new(nuspec: Nuspec, nupkg_path: PathBuf, nupkg_metadata: NupkgMetadata) -> PackageData {
        PackageData {
            id: nuspec.metadata.id,
            version: nuspec.metadata.version,
            source: nupkg_metadata.source,
            nupkg_path,
        }
    }
}

#[derive(Deserialize)]
struct NupkgMetadata {
    source: Url,
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

struct MyCache {
    package_base_address: Cache<String, Url>,
}

impl MyCache {
    fn new() -> Self {
        Self {
            package_base_address: Cache::new(10),
        }
    }
}

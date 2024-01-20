use anyhow::Error;
use dashmap::DashMap;
use serde::Deserialize;
use url::Url;

const NUGET_ORG_INDEX_URL: &str = "https://api.nuget.org/v3/index.json";

pub struct NuGet {
    client: reqwest::Client,
    package_base_address: Url,
    package_cache: DashMap<String, Option<Vec<String>>>,
}

impl NuGet {
    pub async fn new(url: Url) -> anyhow::Result<NuGet> {
        let client = reqwest::Client::new();

        let index: ServiceIndex = client.get(url).send().await?.json().await?;

        let mut package_base_address = index
            .resources
            .into_iter()
            .find(|r| r.typ == "PackageBaseAddress/3.0.0")
            .unwrap()
            .url;

        package_base_address.path_segments_mut().map_err(|_|Error::msg("cannot-be-a-base"))?.pop_if_empty().push("");

        Ok(NuGet {
            client,
            package_base_address,
            package_cache: DashMap::new(),
        })
    }

    pub async fn nuget_org() -> anyhow::Result<NuGet> {
        NuGet::new(Url::parse(NUGET_ORG_INDEX_URL)?).await
    }

    pub async fn exists(&self, package: &str, version: &str) -> bool {
        if !self.package_cache.contains_key(package) {
            async fn get(this: &NuGet, package: &str) -> Option<Vec<String>> {
                let url = this
                    .package_base_address
                    .join(&format!("{}/index.json", package))
                    .ok()?;

                let response: serde_json::Value = this
                    .client
                    .get(url.as_str())
                    .send()
                    .await
                    .ok()?
                    .error_for_status()
                    .ok()?
                    .json()
                    .await
                    .ok()?;

                let vec = if let Some(json_object) = response.as_object() {
                    if json_object.contains_key("versions") {
                        let index: VersionIndex = serde_json::from_value(response).unwrap();
                        index.versions
                    } else {
                        let response: RegistrationResponse = serde_json::from_value(response).unwrap();
                        let leaves = response.items.iter().flat_map(|x| &x.items).collect::<Vec<&RegistrationLeaf>>();
                        let versions = leaves.iter().map(|x| x.catalog_entry.version.to_owned()).collect::<Vec<String>>();
                        versions
                    }
                } else {
                    Vec::new()
                };

                Some(vec)
            }

            self.package_cache
                .insert(package.to_string(), get(self, package).await);
        }

        let cache_entry = self.package_cache.get(package).unwrap();

        cache_entry.is_some()
            && cache_entry
                .as_ref()
                .unwrap()
                .contains(&normalize_version(version).to_string())
    }

    pub fn url(&self, package: &str, version: &str) -> anyhow::Result<Url> {
        Ok(self.package_base_address.join(&format!(
            "{package}/{version}/{package}.{version}.nupkg",
            package = package.to_ascii_lowercase(),
            version = normalize_version(version)
        ))?)
    }
}

fn normalize_version(mut version: &str) -> &str {
    if let Some((ver, _)) = version.split_once('+') {
        version = ver;
    }

    version
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

#[derive(Deserialize)]
struct VersionIndex {
    versions: Vec<String>,
}

#[derive(Deserialize)]
struct RegistrationResponse {
    items: Vec<RegistrationPage>,
}

#[derive(Deserialize)]
struct RegistrationPage {
    items: Vec<RegistrationLeaf>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegistrationLeaf {
    catalog_entry: CatalogEntry,
}

#[derive(Deserialize)]
struct CatalogEntry {
    version: String,
}

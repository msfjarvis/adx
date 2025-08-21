use crate::{package::MavenPackage, version::Version};
use color_eyre::eyre::eyre;
use color_eyre::{Help, Result};
use futures::stream::{self, StreamExt};
use roxmltree::{Document, NodeType};
use semver::Version as Semver;

#[cfg(not(any(test, feature = "nix-check")))]
const BASE_MAVEN_URL: &str = "https://dl.google.com/dl/android/maven2";

/// Downloads the Maven master index for Google's Maven Repository
/// and returns the XML as a String
#[cfg(not(any(test, feature = "nix-check")))]
async fn get_maven_index() -> Result<String> {
    reqwest::get(format!("{BASE_MAVEN_URL}/master-index.xml"))
        .await?
        .text()
        .await
        .map_err(|e| eyre!(e))
}

#[cfg(any(test, feature = "nix-check"))]
#[allow(clippy::unused_async)]
async fn get_maven_index() -> Result<String> {
    std::fs::read_to_string("../testdata/master-index.xml")
        .map_err(|e| eyre!(e).with_note(|| "maven-index".to_string()))
}

/// Downloads the group index for the given group.
#[cfg(not(any(test, feature = "nix-check")))]
async fn get_group_index(group: &str) -> Result<String> {
    reqwest::get(format!(
        "{}/{}/group-index.xml",
        BASE_MAVEN_URL,
        group.replace('.', "/")
    ))
    .await?
    .text()
    .await
    .map_err(|e| eyre!(e))
}

#[cfg(any(test, feature = "nix-check"))]
#[allow(clippy::unused_async)]
async fn get_group_index(group: &str) -> Result<String> {
    std::fs::read_to_string(format!("../testdata/{group}.xml"))
        .map_err(|e| eyre!(e).with_note(|| format!("group_name={group}")))
}

/// Parses a given master-index.xml and filters the found packages based on
// `search_term`.
fn parse_groups<'a>(doc: &'a Document<'_>) -> Vec<&'a str> {
    doc.descendants()
        // Only keep elements
        .filter(|node| node.node_type() == NodeType::Element)
        // Skip the first one since it is junk
        .skip(1)
        .map(|node| node.tag_name())
        .map(|node| node.name())
        .collect()
}

/// Given a list of groups, returns a `Vec<MavenPackage>` of all artifacts.
async fn parse_packages(groups: Vec<&str>) -> Result<Vec<MavenPackage>> {
    // Limit concurrent requests to avoid overwhelming the remote and hitting
    // local resource limits which can lead to silent drops.
    let concurrency: usize = 32;
    let results = stream::iter(groups.into_iter().map(parse_group))
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    Ok(results
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect())
}

/// Given a group, returns a `Vec<MavenPackage>` of all artifacts from this
/// group.
async fn parse_group(group_name: &str) -> Result<Vec<MavenPackage>> {
    let group_index = get_group_index(group_name).await?;
    let doc = Document::parse(&group_index)
        .map_err(|e| eyre!(e).with_note(|| format!("group_name={group_name}")))?;
    Ok(doc
        .descendants()
        .filter(|node| node.node_type() == NodeType::Element)
        .filter(|node| node.tag_name().name() == group_name)
        .flat_map(|node| {
            node.children()
                .filter(|node| node.node_type() == NodeType::Element)
                .filter_map(|node| {
                    let mut versions = node
                        .attribute("versions")
                        .unwrap()
                        .split(',')
                        .filter_map(|v| {
                            if let Ok(semver) = Semver::parse(v) {
                                Some(Version::SemVer(semver))
                            } else {
                                let components: Vec<u16> =
                                    v.split('.').take(3).flat_map(str::parse).collect();
                                if components.len() == 3 {
                                    Some(Version::CalVer((
                                        components[0],
                                        components[1],
                                        components[2],
                                    )))
                                } else {
                                    None
                                }
                            }
                        })
                        .collect::<Vec<Version>>();
                    if versions.is_empty() {
                        None
                    } else {
                        versions.sort_by(|a, b| b.partial_cmp(a).unwrap());
                        Some(MavenPackage {
                            group_id: String::from(group_name),
                            artifact_id: node.tag_name().name().to_string(),
                            versions,
                        })
                    }
                })
                .collect::<Vec<MavenPackage>>()
        })
        .collect())
}

pub(crate) async fn get_packages() -> Result<Vec<MavenPackage>> {
    let maven_index = get_maven_index().await?;
    let doc = Document::parse(&maven_index)?;
    let groups = parse_groups(&doc);
    parse_packages(groups).await
}

/// Fetch and return all group IDs from the master index.
pub(crate) async fn get_groups() -> Result<Vec<String>> {
    let maven_index = get_maven_index().await?;
    let doc = Document::parse(&maven_index)?;
    let groups = parse_groups(&doc);
    Ok(groups
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect())
}

pub(crate) async fn parse(search_term: &str) -> Result<Vec<MavenPackage>> {
    println!("Searching for {search_term}");
    let packages = get_packages().await;
    packages.map(|packages| {
        if search_term.is_empty() {
            packages
        } else {
            packages
                .into_iter()
                .filter(|pkg| {
                    pkg.group_id.contains(search_term) || pkg.artifact_id.contains(search_term)
                })
                .collect()
        }
    })
}

#[cfg(test)]
mod test {
    use super::parse;
    use crate::{channel::Channel, package::LatestPackage};
    use futures::executor::block_on;

    #[test]
    fn check_all_packages_are_parsed() {
        let res = block_on(parse("")).expect("Parsing offline copies should always work");
        let res: Vec<LatestPackage> = res
            .iter()
            .filter_map(|pkg| pkg.latest(Channel::Stable))
            .collect();
        assert_eq!(res.len(), 1684);
    }
}

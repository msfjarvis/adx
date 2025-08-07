use std::convert::TryFrom;

use color_eyre::eyre::eyre;
use color_eyre::{Help, Result};
use futures::future::join_all;
use roxmltree::{Document, NodeType};
use semver::Version as Semver;

use crate::channel::Channel;
use crate::package::MavenPackage;
use crate::version::Version;

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
    std::fs::read_to_string("../testdata/master-index.xml").map_err(|e| eyre!(e))
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
    std::fs::read_to_string(format!("../testdata/{group}.xml")).map_err(|e| eyre!(e))
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
async fn parse_packages(groups: Vec<&str>, channel: Channel) -> Result<Vec<MavenPackage>> {
    // Create a Vec<Future<_>>, this will allow us to run all tasks together
    // without requiring us to spawn a new thread
    let group_futures = groups
        .iter()
        .map(|group_name| parse_group(group_name, channel));

    // Wait for all groups to complete to get a Vec<Vec<MavenPackage>>
    let merged_list = join_all(group_futures).await;

    Ok(merged_list
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect())
}

/// Given a group, returns a `Vec<MavenPackage>` of all artifacts from this
/// group.
async fn parse_group(group_name: &str, channel: Channel) -> Result<Vec<MavenPackage>> {
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
                            if let Ok(sem_ver) = Semver::parse(v) {
                                Some(Version::SemVer(sem_ver))
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
                        .filter(|v| {
                            if let Ok(c) = Channel::try_from(v) {
                                c >= channel
                            } else {
                                false
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
                            latest_version: versions.first().unwrap().to_string(),
                        })
                    }
                })
                .collect::<Vec<MavenPackage>>()
        })
        .collect())
}

pub(crate) async fn parse(search_term: &str, channel: Channel) -> Result<Vec<MavenPackage>> {
    let maven_index = get_maven_index().await?;
    let doc = Document::parse(&maven_index)?;
    let groups = parse_groups(&doc);
    let packages = parse_packages(groups, channel).await;
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
    use super::{Channel, parse};
    use futures::executor::block_on;

    #[test]
    fn check_all_packages_are_parsed() {
        let res = block_on(parse("", Channel::Stable))
            .expect("Parsing offline copies should always work");
        assert_eq!(res.len(), 1367);
    }
}

use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use roxmltree::{Document, NodeType};
use semver::Version;
use std::{
    collections::HashMap,
    convert::TryInto,
    fmt::{Debug, Display, Formatter, Result},
};

/// Struct that represents a Maven package
#[derive(Debug)]
pub(crate) struct MavenPackage {
    pub(crate) group_id: String,
    pub(crate) artifact_id: String,
    pub(crate) latest_version: String,
    pub(crate) all_versions: Vec<Version>,
}

impl Display for MavenPackage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}:{}:{}",
            self.group_id, self.artifact_id, self.latest_version,
        )
    }
}

/// Downloads the Maven master index for Google's Maven Repository
/// and returns the XML as a String
fn get_maven_index() -> anyhow::Result<String> {
    debug!("Downloading maven index...");
    Ok(
        ureq::get("https://dl.google.com/dl/android/maven2/master-index.xml")
            .call()
            .into_string()?,
    )
}

/// Get the group-index.xml URL for a given group
fn get_groups_index_url(group: String) -> String {
    format!(
        "https://dl.google.com/dl/android/maven2/{}/group-index.xml",
        group.replace(".", "/")
    )
}

/// Downloads the group index for a given group, from the given URL.
/// The group parameter is here only for logging purposes and may be removed
/// at any time.
fn get_group_index(group: &str, url: &str) -> anyhow::Result<String> {
    debug!("Getting index for {} from {}", group, url);
    Ok(ureq::get(url).call().into_string()?)
}

/// Parse a given master-index.xml and separate out the AndroidX packages
/// from it.
fn parse_androidx_groups(doc: Document, search_term: &str) -> HashMap<String, String> {
    let mut groups: HashMap<String, String> = HashMap::new();
    for i in doc.descendants() {
        let tag = i.tag_name().name();
        if tag.contains(search_term) {
            groups.insert(tag.to_string(), get_groups_index_url(tag.to_string()));
        }
    }
    groups
}

/// Given a list of groups and a search term to filter against, returns a Vec<MavenPackage>
/// of all artifacts that match the search term.
fn parse_packages(groups: HashMap<String, String>) -> Vec<MavenPackage> {
    let mut packages = Vec::new();
    let pb = ProgressBar::new(groups.len().try_into().unwrap());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{prefix:bold.dim} {spinner} Processing {wide_msg}"),
    );
    for (group_name, group_index_url) in groups.iter() {
        let group_index = get_group_index(group_name, group_index_url)
            .unwrap_or_else(|_| panic!("Failed to get group index for {}", group_name));
        let doc = Document::parse(&group_index)
            .unwrap_or_else(|_| panic!("Failed to parse group index for {}", group_name));
        let mut is_next_root = false;
        let mut group: &str = "";
        for i in doc.descendants() {
            match i.node_type() {
                NodeType::Root => is_next_root = true,
                NodeType::Element => {
                    if is_next_root {
                        group = i.tag_name().name();
                        pb.set_message(group);
                        pb.inc(1);
                        is_next_root = false;
                    } else if !group.is_empty() {
                        let mut versions: Vec<Version> = i
                            .attribute("versions")
                            .unwrap()
                            .split(',')
                            .map(|v| {
                                // This will appear completely nonsensical at first, but I assure you it is not.
                                // The semver crate only accepts versions that contain at least 3 decimal points,
                                // because the semver specification says they must be major.minor.patch . However,
                                // In a critical failure of judgement, the AndroidX team published core-ktx with
                                // invalid semver for 3 releases: 0.1, 0.2, and 0.3. Since maven artifacts are
                                // supposed to be set in stone, we can't make them go back and change those, hence
                                // resorting to this monstrosity that in the end simply counts the number of periods
                                // in the version string, and adds a '.0' as suffix if there are less than 2 of them.
                                if v.chars().filter(|c| c == &'.').count() < 2 {
                                    Version::parse(&format!("{}.0", v))
                                } else {
                                    Version::parse(v)
                                }
                            })
                            // Only take values that were correctly parsed
                            .take_while(|x| x.is_ok())
                            // Unwrap values that were previously determined to be safe
                            .map(|x| x.unwrap())
                            .collect();
                        if versions.is_empty() {
                            continue;
                        }
                        versions.sort_by(|a, b| b.partial_cmp(a).unwrap());
                        packages.push(MavenPackage {
                            group_id: String::from(group),
                            artifact_id: i.tag_name().name().to_string(),
                            latest_version: versions.get(0).unwrap().to_string(),
                            all_versions: versions,
                        })
                    }
                }
                _ => (),
            }
        }
    }
    pb.finish_and_clear();
    packages
}

/// The entrypoint for this module which handles outputting the final result.
pub(crate) fn parse(search_term: &str) -> anyhow::Result<Vec<MavenPackage>> {
    let maven_index = get_maven_index().expect("Failed to get master maven index");
    let doc = Document::parse(&maven_index).expect("Failed to parse master maven index");
    let groups = parse_androidx_groups(doc, search_term);
    let packages = parse_packages(groups);
    Ok(packages)
}

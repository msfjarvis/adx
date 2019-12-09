use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::result::Result;

use indicatif::ProgressBar;
use log::info;
use roxmltree::Document;
use roxmltree::NodeType;

/// Struct that represents a Maven package
pub struct MavenPackage {
    group_id: String,
    artifact_id: String,
    all_versions: Vec<String>,
}

impl fmt::Debug for MavenPackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.group_id, self.artifact_id, self.all_versions[0]
        )
    }
}

impl fmt::Display for MavenPackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Group ID: {}\nArtifact ID: {}\nAvailable versions: {}\nLatest: {}:{}:{}\n",
            self.group_id,
            self.artifact_id,
            self.all_versions.join(", "),
            self.group_id,
            self.artifact_id,
            self.all_versions[0]
        )
    }
}

#[cfg(test)]
fn get_maven_index() -> Result<String, std::io::Error> {
    info!("Reading maven index from disk");
    std::fs::read_to_string("offline-copy/master-index.xml")
}

/// Downloads the Maven master index for Google's Maven Repository
/// and returns the XML as a String
#[cfg(not(test))]
fn get_maven_index() -> Result<String, std::io::Error> {
    info!("Downloading maven index...");
    ureq::get("https://dl.google.com/dl/android/maven2/master-index.xml")
        .call()
        .into_string()
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
#[cfg(not(test))]
fn get_group_index(group: &str, url: &str) -> Result<String, std::io::Error> {
    info!("Getting index for {} from {}", group, url);
    ureq::get(url).call().into_string()
}

#[cfg(test)]
fn get_group_index(group: &str, _: &str) -> Result<String, std::io::Error> {
    info!("Reading group index for {} from disk", group);
    std::fs::read_to_string(format!("offline-copy/{}/group-index.xml", group))
}

/// Parse a given master-index.xml and separate out the AndroidX packages
/// from it.
fn parse_androidx_groups(doc: Document, search_term: String) -> HashMap<String, String> {
    let mut groups: HashMap<String, String> = HashMap::new();
    for i in doc.descendants() {
        let tag = i.tag_name().name();
        if tag.starts_with("androidx") && tag.contains(&search_term) {
            groups.insert(tag.to_string(), get_groups_index_url(tag.to_string()));
        }
    }
    groups
}

/// Given a list of groups and a search term to filter against, returns a Vec<MavenPackage>
/// of all artifacts that match the search term.
fn parse_packages(groups: HashMap<String, String>) -> Vec<MavenPackage> {
    let mut packages = Vec::new();
    let pb: Option<ProgressBar> = match cfg!(debug_assertions) {
        true => None,
        false => Some(ProgressBar::new(groups.len().try_into().unwrap())),
    };
    for (group_name, group_index_url) in groups.iter() {
        let group_index = get_group_index(group_name, group_index_url)
            .expect(&format!("Failed to get group index for {}", group_name));
        let doc = Document::parse(&group_index)
            .expect(&format!("Failed to parse group index for {}", group_name));
        let mut is_next_root = false;
        let mut group: &str = "";
        for i in doc.descendants() {
            match i.node_type() {
                NodeType::Root => is_next_root = true,
                NodeType::Element => {
                    if is_next_root {
                        group = i.tag_name().name();
                        is_next_root = false;
                    } else if !group.is_empty() {
                        let mut versions: Vec<String> = i
                            .attribute("versions")
                            .unwrap()
                            .split(',')
                            .map(|v| v.to_string())
                            .collect();
                        versions.reverse();
                        packages.push(MavenPackage {
                            group_id: String::from(group),
                            artifact_id: i.tag_name().name().to_string(),
                            all_versions: versions,
                        })
                    }
                }
                _ => (),
            }
        }
        if pb.is_some() {
            pb.clone().unwrap().inc(1);
        }
    }
    if pb.is_some() {
        pb.unwrap().finish_and_clear();
    }
    packages
}

/// The entrypoint for this module which handles outputting the final result.
pub fn parse(search_term: String) -> Result<Vec<MavenPackage>, Box<dyn std::error::Error>> {
    let maven_index = get_maven_index().expect("Failed to get master maven index");
    let doc = Document::parse(&maven_index).expect("Failed to parse master maven index");
    let groups = parse_androidx_groups(doc, search_term);
    let packages = parse_packages(groups);
    Ok(packages)
}

#[cfg(test)]
mod test {
    use super::parse;

    #[test]
    fn check_filter_works() {
        let res =
            parse(String::from("appcompat")).expect("Parsing offline copies should always work");
        assert_eq!(res.len(), 2);
        assert!(res.get(0).unwrap().group_id.contains("appcompat"));
        assert!(res.get(1).unwrap().group_id.contains("appcompat"));
    }

    #[test]
    fn check_all_packages_are_parsed() {
        let res = parse(String::new()).expect("Parsing offline copies should always work");
        assert_eq!(res.len(), 211);
    }
}

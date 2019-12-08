#[cfg(not(test))]
use log::info;
#[cfg(not(test))]
use reqwest::get;
#[cfg(not(test))]
use reqwest::Error;
use roxmltree::Document;
use roxmltree::NodeType;
use std::collections::HashMap;
use std::fmt;
use std::result::Result;

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
    std::fs::read_to_string("offline-copy/master-index.xml")
}

/// Downloads the Maven master index for Google's Maven Repository
/// and returns the XML as a String
#[cfg(not(test))]
fn get_maven_index() -> Result<String, Error> {
    info!("Downloading maven index...");
    get("https://dl.google.com/dl/android/maven2/master-index.xml")?.text()
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
fn get_group_index(group: &str, url: &str) -> Result<String, Error> {
    info!("Getting index for {} from {}", group, url);
    get(url)?.text()
}

#[cfg(test)]
fn get_group_index(group: &str, _: &str) -> Result<String, std::io::Error> {
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
    for (group_name, group_index_url) in groups.iter() {
        let group_index = get_group_index(group_name, group_index_url).unwrap();
        let doc = Document::parse(group_index.as_str()).unwrap();
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
                            .collect::<Vec<&str>>()
                            .iter()
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
    }
    packages
}

/// The entrypoint for this module which handles outputting the final result.
pub fn parse(search_term: String) -> Result<Vec<MavenPackage>, Box<dyn std::error::Error>> {
    let maven_index = get_maven_index().unwrap();
    let doc = Document::parse(maven_index.as_str()).unwrap();
    let groups = parse_androidx_groups(doc, search_term);
    let packages = parse_packages(groups);
    Ok(packages)
}

#[cfg(test)]
mod test {
    use super::parse;

    #[test]
    fn check_filter_works() {
        let res = parse(String::from("appcompat")).unwrap();
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn check_all_packages_are_parsed() {
        let res = parse(String::new()).unwrap();
        assert_eq!(res.len(), 211);
    }
}

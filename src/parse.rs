use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::result::Result;

use crate::channel::Channel;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use log::debug;
use roxmltree::Document;
use roxmltree::NodeType;

/// Struct that represents a Maven package
pub struct MavenPackage {
    group_id: String,
    artifact_id: String,
    all_versions: Vec<String>,
    latest_dev: Option<String>,
    latest_alpha: Option<String>,
    latest_beta: Option<String>,
    latest_rc: Option<String>,
    latest_stable: Option<String>,
}

impl MavenPackage {
    pub(crate) fn get_condensed(&self) -> String {
        let mut result = String::new();
        let version = vec![
            self.latest_stable.as_ref(),
            self.latest_rc.as_ref(),
            self.latest_beta.as_ref(),
            self.latest_alpha.as_ref(),
            self.latest_dev.as_ref(),
        ]
        .iter()
        .find(|x| x.is_some())
        .map(|x| x.unwrap());
        result.push_str(&format!(
            "{}:{}:{}",
            self.group_id,
            self.artifact_id,
            version.unwrap()
        ));
        result
    }
}

impl Display for MavenPackage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Group ID: {}\nArtifact ID: {}\nAvailable versions: {}\nLatest dev: {}\nLatest alpha: {}\nLatest beta: {}\nLatest rc: {}\nLatest stable: {}",
            self.group_id,
            self.artifact_id,
            self.all_versions.join(", "),
            match &self.latest_dev {
                Some(v) => &v,
                None => "none",
            },
            match &self.latest_alpha {
                Some(v) => &v,
                None => "none",
            },
            match &self.latest_beta {
                Some(v) => &v,
                None => "none",
            },
            match &self.latest_rc {
                Some(v) => &v,
                None => "none",
            },
            match &self.latest_stable {
                Some(v) => &v,
                None => "none",
            },
        )
    }
}

/// Downloads the Maven master index for Google's Maven Repository
/// and returns the XML as a String
#[cfg(not(test))]
fn get_maven_index() -> Result<String, std::io::Error> {
    debug!("Downloading maven index...");
    ureq::get("https://dl.google.com/dl/android/maven2/master-index.xml")
        .call()
        .into_string()
}

#[cfg(test)]
fn get_maven_index() -> Result<String, std::io::Error> {
    debug!("Reading maven index from disk");
    std::fs::read_to_string("testdata/master-index.xml")
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
    debug!("Getting index for {} from {}", group, url);
    ureq::get(url).call().into_string()
}

#[cfg(test)]
fn get_group_index(group: &str, _: &str) -> Result<String, std::io::Error> {
    debug!("Reading group index for {} from disk", group);
    std::fs::read_to_string(format!("testdata/{}/group-index.xml", group))
}

/// Parse a given master-index.xml and separate out the AndroidX packages
/// from it.
fn parse_androidx_groups(doc: Document, search_term: &str) -> HashMap<String, String> {
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
                        let mut versions: Vec<String> = i
                            .attribute("versions")
                            .unwrap()
                            .split(',')
                            .map(|v| v.to_string())
                            .collect();
                        versions.reverse();
                        let cloned = versions.clone();
                        let stable = cloned
                            .iter()
                            .find(|v| Channel::from_version(v) == Channel::Stable);
                        let rc = cloned
                            .iter()
                            .find(|v| Channel::from_version(v) == Channel::RC);
                        let beta = cloned
                            .iter()
                            .find(|v| Channel::from_version(v) == Channel::Beta);
                        let alpha = cloned
                            .iter()
                            .find(|v| Channel::from_version(v) == Channel::Alpha);
                        let dev = cloned
                            .iter()
                            .find(|v| Channel::from_version(v) == Channel::Dev);
                        packages.push(MavenPackage {
                            group_id: String::from(group),
                            artifact_id: i.tag_name().name().to_string(),
                            all_versions: versions,
                            latest_dev: match dev {
                                Some(s) => Some(String::from(s)),
                                None => None,
                            },
                            latest_alpha: match alpha {
                                Some(s) => Some(String::from(s)),
                                None => None,
                            },
                            latest_beta: match beta {
                                Some(s) => Some(String::from(s)),
                                None => None,
                            },
                            latest_rc: match rc {
                                Some(s) => Some(String::from(s)),
                                None => None,
                            },
                            latest_stable: match stable {
                                Some(s) => Some(String::from(s)),
                                None => None,
                            },
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
pub fn parse(search_term: &str) -> Result<Vec<MavenPackage>, Box<dyn std::error::Error>> {
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
        let res = parse("appcompat").expect("Parsing offline copies should always work");
        assert_eq!(res.len(), 2);
        assert!(res.get(0).unwrap().group_id.contains("appcompat"));
        assert!(res.get(1).unwrap().group_id.contains("appcompat"));
    }

    #[test]
    fn check_all_packages_are_parsed() {
        let res = parse("").expect("Parsing offline copies should always work");
        assert_eq!(res.len(), 212);
    }

    #[test]
    fn channels_are_found_correctly() {
        let mut res = parse("appcompat").expect("Parsing offline copies should always work");
        if let Some(package) = res.get(0) {
            assert!(package.latest_dev == None);
            assert!(package.latest_alpha == Some(String::from("1.2.0-alpha01")));
            assert!(package.latest_beta == Some(String::from("1.1.0-beta01")));
            assert!(package.latest_rc == Some(String::from("1.1.0-rc01")));
            assert!(package.latest_stable == Some(String::from("1.1.0")));
        }
        res = parse("compose").expect("Parsing offline copies should always work");
        if let Some(package) = res.get(0) {
            assert!(package.latest_dev == Some(String::from("0.1.0-dev03")));
            assert!(package.latest_alpha == None);
            assert!(package.latest_beta == None);
            assert!(package.latest_rc == None);
            assert!(package.latest_stable == None);
        }
    }
}

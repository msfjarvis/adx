use roxmltree::Document;
use roxmltree::NodeType;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::result::Result;

struct MavenPackage {
    group_id: String,
    artifact_id: String,
    all_versions: Box<Vec<String>>,
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

#[cfg(debug_assertions)]
fn get_maven_index() -> String {
    std::fs::read_to_string("offline-copy/master-index.xml").unwrap()
}

#[cfg(not(debug_assertions))]
fn get_maven_index() -> String {
    reqwest::get("https://dl.google.com/dl/android/maven2/master-index.xml")?
        .text()
        .unwrap()
}

fn get_groups_index_url(group: String) -> String {
    format!(
        "https://dl.google.com/dl/android/maven2/{}/group-index.xml",
        group.replace(".", "/")
    )
}

#[cfg(not(debug_assertions))]
fn get_group_index(group: &str, url: &str) -> String {
    reqwest::get(url)?.text().unwrap()
}

#[cfg(debug_assertions)]
fn get_group_index(group: &str, _: &str) -> String {
    std::fs::read_to_string(format!("offline-copy/{}/group-index.xml", group)).unwrap()
}

fn parse_groups(doc: Document) -> HashMap<String, String> {
    let mut groups: HashMap<String, String> = HashMap::new();
    for i in doc.descendants() {
        let tag = i.tag_name().name();
        if tag.starts_with("androidx") {
            groups.insert(tag.to_string(), get_groups_index_url(tag.to_string()));
        }
    }
    groups
}

fn parse_packages(groups: HashMap<String, String>, search_term: String) -> Vec<MavenPackage> {
    let mut packages = Vec::new();
    for (group_name, group_index_url) in groups.iter() {
        let group_index = get_group_index(group_name, group_index_url);
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
                    } else if !group.is_empty() && i.tag_name().name().contains(&search_term) {
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
                            all_versions: Box::from(versions),
                        })
                    }
                }
                _ => (),
            }
        }
    }
    packages
}

pub fn parse(search_term: String, detailed_view: bool) -> Result<(), Box<dyn Error>> {
    let maven_index = get_maven_index();
    let doc = Document::parse(maven_index.as_str()).unwrap();
    let groups = parse_groups(doc);
    let packages = parse_packages(groups, search_term);
    if detailed_view {
        for package in packages.iter() {
            println!("{}", package);
        }
    } else {
        for package in packages.iter() {
            println!("{:?}", package);
        }
    }
    Ok(())
}

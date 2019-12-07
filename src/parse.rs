use roxmltree::Document;
use roxmltree::NodeType;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::result::Result;

struct MavenPackage {
    group_id: String,
    artifact_id: String,
    latest_version: String,
}

impl fmt::Display for MavenPackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.group_id, self.artifact_id, self.latest_version
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

fn get_groups_index_url(group: &str) -> String {
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

pub fn parse_packages(search_term: &str) -> Result<(), Box<dyn Error>> {
    let mut groups: HashMap<&str, String> = HashMap::new();
    let mut packages: Vec<MavenPackage> = Vec::new();
    let maven_index = get_maven_index();
    let doc = Document::parse(maven_index.as_str()).unwrap();
    for i in doc.descendants() {
        let tag = i.tag_name().name();
        if tag.starts_with("androidx") {
            groups.insert(tag, get_groups_index_url(tag));
        }
    }
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
                    } else if !group.is_empty() && i.tag_name().name().contains(search_term) {
                        let versions = i
                            .attribute("versions")
                            .unwrap()
                            .split(',')
                            .collect::<Vec<&str>>();
                        packages.push(MavenPackage {
                            group_id: String::from(group),
                            artifact_id: i.tag_name().name().to_string(),
                            latest_version: String::from(versions[versions.len() - 1]),
                        })
                    }
                }
                _ => (),
            }
        }
    }
    for package in packages.iter() {
        println!("{}", package);
    }
    Ok(())
}

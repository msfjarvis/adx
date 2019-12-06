use log::info;
use roxmltree::Document;
use roxmltree::NodeType;
use std::collections::HashMap;
use std::error::Error;
use std::result::Result;

struct MavenPackage {
    artifact_name: String,
    latest_version: String,
}

#[cfg(debug_assertions)]
fn get_maven_index() -> String {
    info!("Using offline maven index");
    std::fs::read_to_string("offline-copy/master-index.xml").unwrap()
}

#[cfg(not(debug_assertions))]
fn get_maven_index() -> String {
    info!("Fetching maven index from the internet");
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
    info!("Fetching group index for {} from the internet", group);
    reqwest::get(url)?.text().unwrap()
}

#[cfg(debug_assertions)]
fn get_group_index(group: &str, _: &str) -> String {
    info!("Fetching group index for {} from local disk", group);
    std::fs::read_to_string(format!("offline-copy/{}/group-index.xml", group)).unwrap()
}

pub fn parse_packages() -> Result<(), Box<dyn Error>> {
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
                    } else if !group.is_empty() {
                        let versions = i.attribute("versions").unwrap().split(",").collect::<Vec<&str>>();
                        packages.push(MavenPackage {
                            artifact_name: format!("{}:{}", group, i.tag_name().name()),
                            latest_version: String::from(versions[versions.len() - 1])
                        })
                    }
                },
                _ => (),
            }
        }
    }
    for package in packages.iter() {
        println!("{}:{}", package.artifact_name, package.latest_version);
    }
    Ok(())
}

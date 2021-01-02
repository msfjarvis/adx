use crate::{channel::Channel, package::MavenPackage};
use color_eyre::{eyre::WrapErr, Result};
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use roxmltree::{Document, NodeType};
use semver::Version;
use std::convert::{TryFrom, TryInto};

/// Downloads the Maven master index for Google's Maven Repository
/// and returns the XML as a String
fn get_maven_index() -> Result<String> {
    debug!("Downloading maven index...");
    Ok(
        ureq::get("https://dl.google.com/dl/android/maven2/master-index.xml")
            .call()
            .into_string()?,
    )
}

/// Downloads the group index for the given group.
fn get_group_index(group: &str) -> Result<String> {
    let url = format!(
        "https://dl.google.com/dl/android/maven2/{}/group-index.xml",
        group.replace(".", "/")
    );
    Ok(ureq::get(&url).call().into_string()?)
}

/// Parses a given master-index.xml and filters the found packages based on
// `search_term`.
fn filter_groups(doc: Document, search_term: &str) -> Vec<String> {
    let mut groups = vec![];
    for i in doc.descendants() {
        let tag = i.tag_name().name();
        if tag.contains(search_term) {
            groups.push(tag.to_string());
        }
    }
    groups
}

/// Given a list of groups, returns a `Vec<MavenPackage>` of all artifacts.
fn parse_packages(groups: Vec<String>, channel: Channel) -> Result<Vec<MavenPackage>> {
    let mut packages = Vec::new();
    let pb = ProgressBar::new(groups.len().try_into().unwrap());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{prefix:bold.dim} {spinner} Processing {wide_msg}"),
    );
    for group_name in groups.iter() {
        let group_index = get_group_index(group_name)
            .context(format!("Failed to get group index for {}", group_name))?;
        let doc = Document::parse(&group_index)
            .context(format!("Failed to parse group index for {}", group_name))?;
        let mut is_next_root = false;
        let mut group: &str = "";
        doc.descendants().for_each(|node| match node.node_type() {
            NodeType::Root => is_next_root = true,
            NodeType::Element => {
                if is_next_root {
                    group = node.tag_name().name();
                    pb.set_message(group);
                    pb.inc(1);
                    is_next_root = false;
                } else if !group.is_empty() {
                    let mut versions: Vec<Version> = node
                        .attribute("versions")
                        .unwrap()
                        .split(',')
                        .map(|v| Version::parse(v))
                        // Only take values that were correctly parsed
                        .take_while(|x| x.is_ok())
                        // Unwrap values that were previously determined to be safe
                        .map(|x| x.unwrap())
                        .collect();
                    // TODO(msfjarvis): Replace when drain_filter becomes stable
                    // https://github.com/rust-lang/rust/issues/43244
                    let channel_filter = |x: &Version| {
                        if let Ok(c) = Channel::try_from(x.to_owned()) {
                            c < channel
                        } else {
                            true
                        }
                    };
                    let mut i = 0;
                    while i != versions.len() {
                        if channel_filter(&versions[i]) {
                            versions.remove(i);
                        } else {
                            i += 1;
                        }
                    }
                    if !versions.is_empty() {
                        versions.sort_by(|a, b| b.partial_cmp(a).unwrap());
                        packages.push(MavenPackage {
                            group_id: String::from(group),
                            artifact_id: node.tag_name().name().to_string(),
                            latest_version: versions.get(0).unwrap().to_string(),
                        })
                    }
                }
            }
            _ => (),
        });
    }
    pb.finish_and_clear();
    Ok(packages)
}

/// The entrypoint for this module which handles outputting the final result.
pub(crate) fn parse(search_term: &str, channel: Channel) -> Result<Vec<MavenPackage>> {
    let maven_index = get_maven_index()?;
    let doc = Document::parse(&maven_index)?;
    let groups = filter_groups(doc, search_term);
    parse_packages(groups, channel)
}

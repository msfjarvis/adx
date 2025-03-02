#!/usr/bin/env python3
"""
Script to vendor a copy of gMaven's indices locally to run tests against
"""

import multiprocessing
import os
import sys
import xml.etree.ElementTree as ET
from concurrent.futures import ThreadPoolExecutor
from urllib.parse import urljoin
import urllib.request
import logging

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

def ensure_directory_exists(directory):
    """Create the directory if it doesn't exist."""
    if not os.path.exists(directory):
        os.makedirs(directory)
        logger.info(f"Created directory: {directory}")
    elif not os.path.isdir(directory):
        logger.error(f"{directory} exists but is not a directory")
        sys.exit(1)

def download_file(url):
    """Download a file from the specified URL."""
    try:
        logger.info(f"Downloading {url}")
        with urllib.request.urlopen(url) as response:
            return response.read()
    except urllib.error.URLError as e:
        logger.error(f"Error downloading {url}: {e}")
        return None

def save_file(content, filepath):
    """Save content to a file."""
    try:
        with open(filepath, 'wb') as f:
            f.write(content)
        logger.info(f"Saved {filepath}")
        return True
    except IOError as e:
        logger.error(f"Error saving {filepath}: {e}")
        return False

def extract_groups(master_index_xml):
    """Extract group names from the master index XML."""
    try:
        root = ET.fromstring(master_index_xml)
        # The group names are the tag names of the children of the root element
        groups = [child.tag for child in root]
        logger.info(f"Found {len(groups)} groups")
        return groups
    except ET.ParseError as e:
        logger.error(f"Error parsing master index XML: {e}")
        return []

def process_group(repo_url, group_name, directory):
    """Download and save a group index."""
    # Replace dots with slashes for the URL path
    group_path = group_name.replace('.', '/')
    group_url = urljoin(repo_url, f"/{group_path}/group-index.xml")
    
    group_content = download_file(group_url)
    if group_content:
        # Save with original group name
        group_filepath = os.path.join(directory, f"{group_name}.xml")
        return save_file(group_content, group_filepath)
    return False

def main():
    """Main function."""
    repo_url = "https://maven.google.com/"
    directory = "testdata"
    max_workers = multiprocessing.cpu_count()
    
    # Ensure the directory exists
    ensure_directory_exists(directory)
    
    # Download master index
    master_index_url = urljoin(repo_url, "/master-index.xml")
    master_index_content = download_file(master_index_url)
    if not master_index_content:
        logger.error("Failed to download master index")
        sys.exit(1)
    
    # Save master index
    master_index_filepath = os.path.join(directory, "master-index.xml")
    if not save_file(master_index_content, master_index_filepath):
        logger.error("Failed to save master index")
        sys.exit(1)
    
    # Extract groups
    groups = extract_groups(master_index_content)
    if not groups:
        logger.error("No groups found in master index")
        sys.exit(1)
    
    # Process each group in parallel
    success_count = 0
    failure_count = 0
    
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        futures = {executor.submit(process_group, repo_url, group, directory): group for group in groups}
        for future in futures:
            group_name = futures[future]
            try:
                if future.result():
                    success_count += 1
                else:
                    logger.warning(f"Failed to process group: {group_name}")
                    failure_count += 1
            except Exception as e:
                logger.error(f"Error processing group {group_name}: {e}")
                failure_count += 1
    
    logger.info(f"Completed. Processed {success_count} groups successfully, {failure_count} failures.")

if __name__ == "__main__":
    main()

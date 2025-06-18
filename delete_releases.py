#!/usr/bin/env python3
"""
Delete GitHub releases script
Deletes all v0.1.*-test releases from the vx repository
"""

import requests
import os
import time

# GitHub API configuration
REPO = "loonghao/vx"
BASE_URL = f"https://api.github.com/repos/{REPO}/releases"
TOKEN = os.environ.get("GITHUB_TOKEN")

# Release IDs to delete (all v0.1.*-test releases)
RELEASE_IDS = [
    "225341819",  # v0.1.32-test
    "225338756",  # vx-v0.1.29-test
    "225337298",  # vx-v0.1.27-test
    "225335132",  # vx-v0.1.25-test
    "225334358",  # vx-v0.1.24-test
    "225328341",  # vx-v0.1.23-test
    "225327670",  # vx-v0.1.22-test
    "225326978",  # vx-v0.1.21-test
    "225326416",  # vx-v0.1.20-test
    "225324748",  # vx-v0.1.19-test
    "225312201",  # vx-v0.1.13-test
    "225308135",  # vx-v0.1.9-test
    "225306567",  # vx-v0.1.5-test
]

def delete_release(release_id):
    """Delete a GitHub release by ID"""
    url = f"{BASE_URL}/{release_id}"
    headers = {
        "Authorization": f"token {TOKEN}",
        "Accept": "application/vnd.github.v3+json",
        "User-Agent": "vx-cleanup-script"
    }
    
    try:
        response = requests.delete(url, headers=headers)
        if response.status_code == 204:
            print(f"✅ Successfully deleted release {release_id}")
            return True
        else:
            print(f"❌ Failed to delete release {release_id}: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print(f"❌ Error deleting release {release_id}: {str(e)}")
        return False

def main():
    if not TOKEN:
        print("❌ GITHUB_TOKEN environment variable not set")
        return
    
    print(f"🗑️ Deleting test releases from {REPO}...")
    print("")
    
    success_count = 0
    for release_id in RELEASE_IDS:
        print(f"Deleting release ID: {release_id}...")
        if delete_release(release_id):
            success_count += 1
        time.sleep(0.5)  # Rate limiting
    
    print("")
    print(f"🎉 Cleanup completed! {success_count}/{len(RELEASE_IDS)} releases deleted successfully.")

if __name__ == "__main__":
    main()

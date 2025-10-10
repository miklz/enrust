#!/usr/bin/env python3

import yaml
import argparse
import re
import requests
import sys
import time
from bs4 import BeautifulSoup

def login(session, base_url, username, password):
    print("🔐 Logging in...")
    login_page = session.get(f"{base_url}/login/")
    csrftoken = session.cookies.get("csrftoken")

    if not csrftoken:
        sys.exit("❌ Could not get CSRF token for login.")

    login_data = {
        "username": username,
        "password": password,
        "csrfmiddlewaretoken": csrftoken,
    }

    headers = {"Referer": f"{base_url}/login/"}
    response = session.post(
        f"{base_url}/login/",
        data=login_data,
        headers=headers,
        allow_redirects=True,
    )

    if "sessionid" not in session.cookies:
        sys.exit("❌ Login failed. Check your username/password.")

    print("✅ Logged in successfully.")
    return session


def create_test(session, base_url, test_data):
    print("🧪 Creating test...")

    # Refresh CSRF token after login
    session.get(f"{base_url}/test/new/")
    csrftoken = session.cookies.get("csrftoken")

    if not csrftoken:
        sys.exit("❌ Could not refresh CSRF token for test creation.")

    headers = {
        "Referer": f"{base_url}/test/new/",
        "X-CSRFToken": csrftoken,
    }

    data = {
        "csrfmiddlewaretoken": csrftoken,
        **test_data,
    }

    response = session.post(
        f"{base_url}/test/new/",
        data=data,
        headers=headers,
        allow_redirects=True,
    )

    print(f"📬 Server returned: {response.status_code}")
    with open("server_response.html", "w") as html_file:
        html_file.write(response.text)

    if response.status_code == 200 and "Finished" in response.text:
        print("✅ Test created successfully!")
    elif response.status_code >= 400:
        print("⚠️ Server error. Check response above.")


def get_current_test_ids(session, base_url):
    """Scrape the /index page and extract test IDs from /test/<id>/ links."""
    r = session.get(f"{base_url}/index")
    r.raise_for_status()
    soup = BeautifulSoup(r.text, "html.parser")
    test_links = soup.find_all("a", href=re.compile(r"^/test/\d+/"))
    test_ids = {int(re.search(r"/test/(\d+)/", a["href"]).group(1)) for a in test_links}
    return test_ids


def fetch_new_test(session, base_url, before_ids):
    """Poll /index to find new test IDs."""
    for attempt in range(10):
        time.sleep(3)
        after_ids = get_current_test_ids(session, base_url)
        new_ids = after_ids - before_ids
        if new_ids:
            return max(new_ids)
    raise TimeoutError("No new test appeared after creation.")


def main():
    parser = argparse.ArgumentParser(description="Automate OpenBench test creation")
    parser.add_argument("--config", required=True, help="Path to YAML config file")
    parser.add_argument("--commit", required=True, help="Commit to be tested")
    args = parser.parse_args()

    # Load configuration
    with open(args.config, "r") as f:
        cfg = yaml.safe_load(f)

    base_url = cfg["server"]["base_url"].rstrip("/")
    username = cfg["server"]["username"]
    password = cfg["server"]["password"]
    test_data = cfg["test"]

    # Start session and create test
    with requests.Session() as session:
        login(session, base_url, username, password)
        before = get_current_test_ids(session, base_url)
        create_test(session, base_url, test_data)
        new_test_id = fetch_new_test(session, base_url, before)
        print(f"[OpenBench] Created test {new_test_id} for commit {args.commit}")


if __name__ == "__main__":
    main()


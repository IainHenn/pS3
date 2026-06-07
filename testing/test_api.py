#!/usr/bin/env python3
"""
Integration test script for the pS3 API.

Usage:
    pip install -r testing/requirements.txt
    python testing/test_api.py

Optional env vars:
    API_BASE_URL  default http://localhost:3000
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

import requests

BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:3000").rstrip("/")
FIXTURES_DIR = Path(__file__).parent / "fixtures"


def section(title: str) -> None:
    print(f"\n{'=' * 60}")
    print(title)
    print("=" * 60)


def call(method: str, path: str, **kwargs) -> requests.Response:
    url = f"{BASE_URL}{path}"
    print(f"\n-> {method} {path}")
    response = requests.request(method, url, timeout=30, **kwargs)
    print(f"<- {response.status_code}")

    content_type = response.headers.get("content-type", "")
    if "application/json" in content_type:
        try:
            print(json.dumps(response.json(), indent=2))
        except json.JSONDecodeError:
            print(response.text[:500])
    elif "text/" in content_type or content_type.startswith("application/"):
        preview = response.text[:300] if response.text else f"<{len(response.content)} bytes>"
        print(preview)
    else:
        print(f"<{len(response.content)} bytes>")

    return response


def upload_file(bucket_id: str, file_path: Path, method: str = "POST", file_id: str | None = None) -> requests.Response:
    path = f"/buckets/{bucket_id}/files"
    if method == "PUT":
        if not file_id:
            raise ValueError("file_id is required for PUT uploads")
        path = f"{path}/{file_id}"

    with file_path.open("rb") as handle:
        files = {"file": (file_path.name, handle, "text/plain")}
        return call(method, path, files=files)


def extract_id(response: requests.Response, *keys: str) -> str | None:
    try:
        data = response.json()
    except json.JSONDecodeError:
        return None

    for key in keys:
        if key in data and data[key]:
            return str(data[key])

    if "result" in data and isinstance(data["result"], list) and data["result"]:
        first = data["result"][0]
        if isinstance(first, dict) and "id" in first:
            return str(first["id"])

    return None


def main() -> int:
    if not FIXTURES_DIR.exists():
        print(f"Fixtures directory not found: {FIXTURES_DIR}", file=sys.stderr)
        return 1

    fixture_files = sorted(FIXTURES_DIR.glob("*.txt"))
    if not fixture_files:
        print(f"No .txt fixtures found in {FIXTURES_DIR}", file=sys.stderr)
        return 1

    section("Health")
    health = call("GET", "/health")
    if health.status_code != 200:
        print("API is not healthy. Start the server first.", file=sys.stderr)
        return 1

    section("Buckets - create")
    bucket_a = call("POST", "/buckets", json={"name": "test-bucket-a"})
    bucket_b = call("POST", "/buckets", json={"name": "test-bucket-b"})

    bucket_a_id = extract_id(bucket_a, "id")
    bucket_b_id = extract_id(bucket_b, "id")

    if not bucket_a_id or not bucket_b_id:
        print("Failed to create buckets.", file=sys.stderr)
        return 1

    section("Buckets - get by id")
    call("GET", f"/buckets/{bucket_a_id}")

    section("Buckets - get many")
    call("GET", "/buckets", params={"bucket_ids": f"{bucket_a_id},{bucket_b_id}"})

    section("Buckets - update")
    call("PUT", f"/buckets/{bucket_a_id}", json={"name": "test-bucket-a-renamed"})

    section("Files - create")
    created_file_ids: list[str] = []

    for fixture in fixture_files[:2]:
        response = upload_file(bucket_a_id, fixture, method="POST")
        file_id = extract_id(response, "new_id", "id")
        if file_id:
            created_file_ids.append(file_id)

    if len(created_file_ids) < 2:
        print("Failed to create enough test files.", file=sys.stderr)
        return 1

    file_one_id, file_two_id = created_file_ids[0], created_file_ids[1]

    section("Files - get by id")
    call("GET", f"/buckets/{bucket_a_id}/files/{file_one_id}")

    section("Files - get many")
    call(
        "GET",
        f"/buckets/{bucket_a_id}/files",
        params={"file_ids": f"{file_one_id},{file_two_id}"},
    )

    section("Files - update")
    upload_file(
        bucket_a_id,
        fixture_files[2],
        method="PUT",
        file_id=file_one_id,
    )

    section("Files - delete one")
    call("DELETE", f"/buckets/{bucket_a_id}/files/{file_two_id}")

    section("Files - delete many")
    call(
        "DELETE",
        f"/buckets/{bucket_a_id}/files",
        params={"file_ids": file_one_id},
    )

    section("Buckets - delete one")
    call("DELETE", f"/buckets/{bucket_b_id}")

    section("Buckets - delete many")
    call("DELETE", "/buckets", params={"bucket_ids": bucket_a_id})

    section("Done")
    print("All API routes were exercised.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

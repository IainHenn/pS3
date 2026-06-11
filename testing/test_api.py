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
import uuid
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
    if method in ("PUT", "PATCH"):
        if not file_id:
            raise ValueError("file_id is required for PUT/PATCH uploads")
        path = f"{path}/{file_id}"

    with file_path.open("rb") as handle:
        files = {"file": (file_path.name, handle, "text/plain")}
        return call(method, path, files=files)


def move_file(old_bucket_id: str, file_id: str, new_bucket_id: str) -> requests.Response:
    path = f"/buckets/{old_bucket_id}/files/{file_id}/to/{new_bucket_id}"
    return call("PATCH", path)


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

    # Unique names so re-runs don't hit buckets.name UNIQUE from leftover rows.
    run_id = uuid.uuid4().hex[:8]
    bucket_a_name = f"test-bucket-a-{run_id}"
    bucket_b_name = f"test-bucket-b-{run_id}"

    section("Buckets - create")
    bucket_a = call("POST", "/buckets", json={"name": bucket_a_name})
    bucket_b = call("POST", "/buckets", json={"name": bucket_b_name})

    bucket_a_id = extract_id(bucket_a, "id")
    bucket_b_id = extract_id(bucket_b, "id")

    if not bucket_a_id or not bucket_b_id:
        print("Failed to create buckets.", file=sys.stderr)
        if bucket_a.status_code == 500 or bucket_b.status_code == 500:
            print(
                "Server returned 500 — often a duplicate bucket name or DB error. "
                "Check docker/postgres logs.",
                file=sys.stderr,
            )
        return 1

    section("Buckets - get by id")
    call("GET", f"/buckets/{bucket_a_id}")

    section("Buckets - get many")
    call("GET", "/buckets", params={"bucket_ids": f"{bucket_a_id},{bucket_b_id}"})

    section("Buckets - update")
    call("PUT", f"/buckets/{bucket_a_id}", json={"name": f"{bucket_a_name}-renamed"})

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

    section("Files - update (physical content)")
    update_response = upload_file(
        bucket_a_id,
        fixture_files[2],
        method="PATCH",
        file_id=file_one_id,
    )
    if update_response.status_code != 200:
        print("Failed to update file content.", file=sys.stderr)
        return 1

    updated = call("GET", f"/buckets/{bucket_a_id}/files/{file_one_id}")
    if updated.status_code != 200:
        print("Updated file is not readable from original bucket.", file=sys.stderr)
        return 1

    section("Files - move to another bucket")
    move_response = move_file(bucket_a_id, file_one_id, bucket_b_id)
    if move_response.status_code != 200:
        print("Failed to move file between buckets.", file=sys.stderr)
        return 1

    moved = call("GET", f"/buckets/{bucket_b_id}/files/{file_one_id}")
    if moved.status_code != 200:
        print("Moved file is not readable from destination bucket.", file=sys.stderr)
        return 1

    stale = call("GET", f"/buckets/{bucket_a_id}/files/{file_one_id}")
    if stale.status_code != 404:
        print("Moved file should no longer be in the original bucket.", file=sys.stderr)
        return 1

    section("Files - delete one")
    call("DELETE", f"/buckets/{bucket_a_id}/files/{file_two_id}")

    section("Files - delete many")
    call(
        "DELETE",
        f"/buckets/{bucket_b_id}/files",
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

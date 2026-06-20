#!/usr/bin/env python3
"""
Integration test script for the pS3 gRPC API.

Usage:
    pip install -r testing/requirements.txt
    python testing/test_grpc.py

Optional env vars:
    GRPC_TARGET  default localhost:50051
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import uuid
from pathlib import Path

import grpc

ROOT = Path(__file__).resolve().parent.parent
PROTO_DIR = ROOT / "src" / "grpc" / "proto"
GENERATED_DIR = Path(__file__).resolve().parent / "generated"
FIXTURES_DIR = Path(__file__).parent / "fixtures"
GRPC_TARGET = os.environ.get("GRPC_TARGET", "localhost:50051")


def ensure_stubs() -> None:
    GENERATED_DIR.mkdir(parents=True, exist_ok=True)
    init_file = GENERATED_DIR / "__init__.py"
    if not init_file.exists():
        init_file.write_text("", encoding="utf-8")

    proto_path = PROTO_DIR.resolve()
    proto_file = proto_path / "ps3.proto"
    pb2_file = GENERATED_DIR / "ps3_pb2.py"
    if not pb2_file.exists() or pb2_file.stat().st_mtime < proto_file.stat().st_mtime:
        subprocess.run(
            [
                sys.executable,
                "-m",
                "grpc_tools.protoc",
                "--proto_path=.",
                f"--python_out={GENERATED_DIR.resolve()}",
                f"--grpc_python_out={GENERATED_DIR.resolve()}",
                "ps3.proto",
            ],
            check=True,
            cwd=str(proto_path),
        )


ensure_stubs()
sys.path.insert(0, str(GENERATED_DIR))
import ps3_pb2  # noqa: E402
import ps3_pb2_grpc  # noqa: E402


def section(title: str) -> None:
    print(f"\n{'=' * 60}")
    print(title)
    print("=" * 60)


def show(message: str) -> None:
    print(message)


def call(name: str, response) -> None:
    print(f"\n-> {name}")
    print(f"<- {type(response).__name__}")
    print(json.dumps(str(response), indent=2))


def read_file_stream(stub, bucket_id: str, file_id: str) -> bytes:
    stream = stub.GetFileById(
        ps3_pb2.FileRequest(bucket_id=bucket_id, file_id=file_id)
    )
    return b"".join(chunk.content for chunk in stream)


def expect_not_found(stub, bucket_id: str, file_id: str) -> None:
    try:
        read_file_stream(stub, bucket_id, file_id)
    except grpc.RpcError as exc:
        if exc.code() != grpc.StatusCode.NOT_FOUND:
            raise
    else:
        raise AssertionError("expected NOT_FOUND")


def main() -> int:
    if not FIXTURES_DIR.exists():
        print(f"Fixtures directory not found: {FIXTURES_DIR}", file=sys.stderr)
        return 1

    fixture_files = sorted(FIXTURES_DIR.glob("*.txt"))
    if not fixture_files:
        print(f"No .txt fixtures found in {FIXTURES_DIR}", file=sys.stderr)
        return 1

    channel = grpc.insecure_channel(GRPC_TARGET)
    health_stub = ps3_pb2_grpc.HealthStub(channel)
    buckets_stub = ps3_pb2_grpc.BucketsStub(channel)
    files_stub = ps3_pb2_grpc.FilesStub(channel)

    section("Health")
    health = health_stub.GetHealth(ps3_pb2.HealthyRequest())
    call("Health.GetHealth", health)
    if health.message != "Success!":
        print("gRPC health check failed. Start the server first.", file=sys.stderr)
        return 1

    run_id = uuid.uuid4().hex[:8]
    bucket_a_name = f"test-bucket-a-{run_id}"
    bucket_b_name = f"test-bucket-b-{run_id}"

    section("Buckets - create")
    bucket_a = buckets_stub.CreateBucket(
        ps3_pb2.BucketCreateRequest(bucket_name=bucket_a_name)
    )
    bucket_b = buckets_stub.CreateBucket(
        ps3_pb2.BucketCreateRequest(bucket_name=bucket_b_name)
    )
    call("Buckets.CreateBucket A", bucket_a)
    call("Buckets.CreateBucket B", bucket_b)

    bucket_a_id = bucket_a.bucket_id
    bucket_b_id = bucket_b.bucket_id
    if not bucket_a_id or not bucket_b_id:
        print("Failed to create buckets.", file=sys.stderr)
        return 1

    section("Buckets - get by id")
    call(
        "Buckets.GetBucket",
        buckets_stub.GetBucket(ps3_pb2.BucketRequest(bucket_id=bucket_a_id)),
    )

    section("Buckets - get many")
    call(
        "Buckets.GetBuckets",
        buckets_stub.GetBuckets(
            ps3_pb2.BucketsRequest(bucket_ids=f"{bucket_a_id},{bucket_b_id}")
        ),
    )

    section("Buckets - update")
    call(
        "Buckets.UpdateBucket",
        buckets_stub.UpdateBucket(
            ps3_pb2.BucketUpdateRequest(
                bucket_id=bucket_a_id,
                bucket_name=f"{bucket_a_name}-renamed",
            )
        ),
    )

    section("Files - create")
    created_file_ids: list[str] = []
    for fixture in fixture_files[:2]:
        content = fixture.read_bytes()
        response = files_stub.CreateFile(
            ps3_pb2.FileCreateRequest(
                bucket_id=bucket_a_id,
                name=fixture.name,
                mime_type="text/plain",
                content=content,
            )
        )
        call(f"Files.CreateFile {fixture.name}", response)
        if response.new_id:
            created_file_ids.append(response.new_id)

    if len(created_file_ids) < 2:
        print("Failed to create enough test files.", file=sys.stderr)
        return 1

    file_one_id, file_two_id = created_file_ids[0], created_file_ids[1]

    section("Files - get by id")
    downloaded = read_file_stream(files_stub, bucket_a_id, file_one_id)
    show(f"Downloaded {len(downloaded)} bytes from stream")

    section("Files - get many")
    files_reply = files_stub.GetFiles(
        ps3_pb2.FilesRequest(
            bucket_id=bucket_a_id,
            file_ids=f"{file_one_id},{file_two_id}",
        )
    )
    call("Files.GetFiles", files_reply)

    section("Files - update (physical content)")
    update_fixture = fixture_files[2]
    update_response = files_stub.UpdateFile(
        ps3_pb2.FilesUpdateRequest(
            bucket_id=bucket_a_id,
            file_id=file_one_id,
            name=update_fixture.name,
            mime_type="text/plain",
            content=update_fixture.read_bytes(),
        )
    )
    call("Files.UpdateFile", update_response)
    if not update_response.content:
        print("Failed to update file content.", file=sys.stderr)
        return 1

    updated_bytes = read_file_stream(files_stub, bucket_a_id, file_one_id)
    if not updated_bytes:
        print("Updated file is not readable from original bucket.", file=sys.stderr)
        return 1

    section("Files - move to another bucket")
    move_response = files_stub.MoveFile(
        ps3_pb2.FileMovementRequest(
            old_bucket_id=bucket_a_id,
            new_bucket_id=bucket_b_id,
            file_id=file_one_id,
        )
    )
    call("Files.MoveFile", move_response)
    if not move_response.result:
        print("Failed to move file between buckets.", file=sys.stderr)
        return 1

    moved_bytes = read_file_stream(files_stub, bucket_b_id, file_one_id)
    if not moved_bytes:
        print("Moved file is not readable from destination bucket.", file=sys.stderr)
        return 1

    try:
        expect_not_found(files_stub, bucket_a_id, file_one_id)
    except AssertionError:
        print("Moved file should no longer be in the original bucket.", file=sys.stderr)
        return 1

    section("Files - delete one")
    call(
        "Files.DeleteFile",
        files_stub.DeleteFile(
            ps3_pb2.FileDeleteRequest(bucket_id=bucket_a_id, file_id=file_two_id)
        ),
    )

    section("Files - delete many")
    call(
        "Files.DeleteFiles",
        files_stub.DeleteFiles(
            ps3_pb2.FilesDeleteRequest(bucket_id=bucket_b_id, file_ids=file_one_id)
        ),
    )

    section("Buckets - delete one")
    call(
        "Buckets.DeleteBucket",
        buckets_stub.DeleteBucket(ps3_pb2.BucketDeleteRequest(bucket_id=bucket_b_id)),
    )

    section("Buckets - delete many")
    call(
        "Buckets.DeleteBuckets",
        buckets_stub.DeleteBuckets(
            ps3_pb2.BucketDeletesRequest(bucket_ids=bucket_a_id)
        ),
    )

    section("Done")
    print("All gRPC routes were exercised.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

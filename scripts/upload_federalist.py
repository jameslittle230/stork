import os
import sys
import boto3
import time

if "AWS_ACCESS_KEY_ID" not in os.environ or "AWS_SECRET_ACCESS_KEY" not in os.environ:
    print("Error: Environment variables `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` must be set in order to upload to AWS S3.")
    exit(1)

normalized_dir = filedir = os.path.dirname(os.path.realpath(__file__))
opj = os.path.join

def uploadFile(localPath, remotePath, extraArgs={}):
    print(f"Called uploadFile: {localPath} → {remotePath}")

    s3 = boto3.resource('s3')
    s3.Bucket("files.stork-search.net").upload_file(localPath, remotePath, ExtraArgs=extraArgs)


def invalidate():
    cloudfront = boto3.client("cloudfront")
    cloudfront.create_invalidation(
        DistributionId="E3PBNOZP9XRSWN",
        InvalidationBatch={
            'Paths': {
                'Quantity': 1,
                'Items': [
                    '/*',
                ]
            },
            'CallerReference': f"{time.time()}"
        }
    )


if __name__ == "__main__":
    web_artifacts = [
        {"filename": "stork.js", "contentType": "text/javascript"},
        {"filename": "stork.wasm", "contentType": "application/wasm"},
        {"filename": "basic.css", "contentType": "text/css"},
        {"filename": "dark.css", "contentType": "text/css"},
    ]

    binaries = [
        "stork-macos-10-15",
        "stork-ubuntu-20-04",
    ]

    other_files = [
        "federalist.st"
    ]

    ref = sys.argv[1] # We'll upload to /releases/${ref}/*

    print(f"Uploading files...")

    for file in web_artifacts:
        for destination_path in [
           opj("releases", ref, file["filename"]),
           opj("releases", "latest", file["filename"]),
           file["filename"]
        ]:
            source_path = opj(normalized_dir, "..", "web-artifacts", file["filename"])

            uploadFile(source_path, destination_path, {'ContentType': file["contentType"]})

    for binary in binaries:
        for destination_path in [
            opj("releases", ref, binary),
            opj("releases", "latest", binary),
        ]:
            source_path = opj(normalized_dir, "..", binary, "stork")
            uploadFile(source_path, destination_path)

    for file in other_files:
        for destination_path in [
            opj("releases", ref, file),
            opj("releases", "latest", file),
            file
        ]:
            source_path = opj(normalized_dir, "..", file)
            uploadFile(source_path, destination_path)

    invalidate()
    print("Cache invalidated.")
    print("Done. Visit https://stork-search.net")

import os
import sys
# import boto3
import time

# REQUIREMENTS:
# This script should _only_ be run as part of the create-release-after-tag
# Github action. It requires build artifacts to be in certain locations
# (see comments below) that only this Github action can properly set up.


def uploadFile(localPath, remotePath, extraArgs={}):
    print(f"Called uploadFile: {localPath} → {remotePath}")

    s3 = boto3.resource('s3')
    s3.Bucket("files.stork-search.net").upload_file(localPath,
                                                    remotePath, ExtraArgs=extraArgs)


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

    opj = os.path.join

    if "GITHUB_ACTIONS" not in os.environ or os.environ['GITHUB_ACTIONS'] is False:
        print("WARNING: Environment variable `GITHUB_ACTIONS` not present.\nYou likely are misusing this script -- This script should _only_ be run\nas part of the create-release-after-tag Github action. Exiting.")
        exit(1)

    if not os.path.exists(opj(os.getcwd(), ".stork-project-root")):
        print(
            f"Current working directory {os.getcwd()} doesn't look to be the Stork project root.\nRun this as `just upload` or run it from the Stork root directory. Exiting.")
        exit(1)

    projroot = os.getcwd()
    ref = sys.argv[1]  # Script takes one command line argument

    if not ref or len(ref) < 1:
        print("No argument passed to this script. You must pass an argument which will become the directory to which files are uploaded on the CDN.")
        print("Usage: python3 scripts/upload_federalist.py \"v1.2.5\"")
        exit(1)

    if "AWS_ACCESS_KEY_ID" not in os.environ or "AWS_SECRET_ACCESS_KEY" not in os.environ:
        print("Error: Environment variables `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` must be set in order to upload to AWS S3.")
        exit(1)

    # Script expects that these files will all be present in
    # the ./web-artifacts directory in the project root.
    web_artifacts = [
        {"filename": "stork.js", "contentType": "text/javascript"},
        {"filename": "stork.wasm", "contentType": "application/wasm"},
        {"filename": "stork.js.map", "contentType": "binary/octet-stream"},
        {"filename": "basic.css", "contentType": "text/css"},
        {"filename": "dark.css", "contentType": "text/css"},
        {"filename": "flat.css", "contentType": "text/css"},
        {"filename": "edible.css", "contentType": "text/css"},
        {"filename": "edible-dark.css", "contentType": "text/css"},
    ]

    # Script expects that for each file below, a corresponding file
    # will exist at ./{binary}/stork in the project root.
    binaries = [
        "stork-macos-10-15",
        "stork-ubuntu-20-04",
    ]

    # Script expects that these files will exist in the project root.
    other_files = [
        "federalist.st",
    ]

    ref = sys.argv[1]  # We'll upload to /releases/${ref}/*

    print(f"Uploading {len(web_artifacts) + len(binaries) + len(other_files)} files to files.stork-search.net/releases/{ref} ...")

    for file in web_artifacts:

        for destination_path in [
            opj("releases", ref, file["filename"]),
            opj("releases", "latest", file["filename"]),
        ]:
            source_path = opj(projroot, "web-artifacts", file["filename"])

            uploadFile(source_path, destination_path, {
                       'ContentType': file["contentType"]})

    for binary in binaries:
        for destination_path in [
            opj("releases", ref, binary),
            opj("releases", "latest", binary),
        ]:
            source_path = opj(projroot, binary, "stork")
            uploadFile(source_path, destination_path)

    for file in other_files:
        for destination_path in [
            opj("releases", ref, file),
            opj("releases", "latest", file),
        ]:
            source_path = opj(projroot, file)
            uploadFile(source_path, destination_path)

    invalidate()
    print("Cache invalidated.")

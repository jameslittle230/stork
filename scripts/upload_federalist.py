import os
import boto3
import time

if "AWS_ACCESS_KEY_ID" not in os.environ or "AWS_SECRET_ACCESS_KEY" not in os.environ:
    print("Error: Environment variables `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` must be set in order to upload to AWS S3.")
    exit(1)

normalized_dir = filedir = os.path.dirname(os.path.realpath(__file__))
opj = os.path.join


def uploadFile(localPath, remotePath, bucket, extraArgs={}):
    s3 = boto3.resource('s3')
    s3.Bucket(bucket).upload_file(localPath, remotePath, ExtraArgs=extraArgs)


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
    dist_files = [
        {"filename": "stork.js", "contentType": "text/javascript"},
        {"filename": "stork.wasm", "contentType": "application/wasm"},
        {"filename": "federalist.st", "contentType": "application/octet-stream"},
        {"filename": "basic.css", "contentType": "text/css"},
        {"filename": "dark.css", "contentType": "text/css"},
    ]

    binaries = [
        "stork-macos-10-15", 
        "stork-ubuntu-20-04",
        "stork-ubuntu-16-04",
        "stork-windows-2019"
    ]

    ref = argv[2] # We'll upload to /releases/${ref}/*

    outof = len(dist_files) + len(binaries)
    idx = 0

    print(f"Uploading {outof} files...")

    for file in dist_files:
        idx += 1

        uploadFile(opj(normalized_dir, "..", "dist", file["filename"]),
                   opj("releases", ref, file["filename"]),
                   "files.stork-search.net",
                   {'ContentType': file["contentType"]})

        uploadFile(opj(normalized_dir, "..", "dist", file["filename"]),
                   opj("releases", "latest", file["filename"]),
                   "files.stork-search.net",
                   {'ContentType': file["contentType"]})
        
        uploadFile(opj(normalized_dir, "..", "dist", file["filename"]),
                   file["filename"],
                   "files.stork-search.net",
                   {'ContentType': file["contentType"]})
        
        print(f"{idx} of {outof} files uploaded to S3.")
    
    for binary in binaries:
        idx += 1

        uploadFile(opj(normalized_dir, "..", binary),
                   opj("releases", ref, binary),
                   "files.stork-search.net")

        uploadFile(opj(normalized_dir, "..", binary),
                   opj("releases", "latest", binary),
                   "files.stork-search.net")
        
        print(f"{idx} of {outof} files uploaded to S3.")

    invalidate()
    print("Cache invalidated.")
    print("Done. Visit https://stork-search.net")

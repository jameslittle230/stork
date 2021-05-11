import os
import sys
import json
import subprocess

# Step 1: Build the project from scratch
subprocess.run(["./scripts/build.sh"])

# Step 2: get file sizes for various distributed files
files = [
    './dist/federalist.st',
    './dist/stork.wasm',
    './dist/stork.js'
]

sizes = dict([(file.split('./dist/')[1], float(os.path.getsize(file))/1000) for file in files])

# Step 3: Run benchmarks and get mean runtime for each
benchmarks = [
    "build/federalist",
    "search/federalist/liberty"
]

for bench_name in benchmarks:
    print(f"Running benchmark for {bench_name}", file=sys.stderr)
    run_bench_cmd = subprocess.run(
        ["cargo", "criterion", "--message-format=json", bench_name],
        stdout=subprocess.PIPE,
        text=True
    )

    grep_for_success_cmd = subprocess.run(
        ["grep", "benchmark-complete"],
        input=run_bench_cmd.stdout,
        stdout=subprocess.PIPE,
        text=True
    )

    jq_cmd = subprocess.run(
        ["jq", ".mean.estimate / 1000000"],
        input=grep_for_success_cmd.stdout,
        capture_output=True,
        text=True
    )

    bench_time_ms = float(jq_cmd.stdout)

    # Step 4: Print out results
    sizes.update({
        bench_name: bench_time_ms
    })

print(json.dumps(sizes, indent=2))
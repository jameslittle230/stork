import os
import subprocess
import json

# Step 1: Build the project from scratch
subprocess.run(["./scripts/build.sh"])

# Step 2: get file sizes for various distributed files
files = [
    './dist/federalist.st',
    './dist/stork.wasm',
    './dist/stork.js'
]

sizes = dict([(file.split('./dist/')[1], float(os.path.getsize(file))/1000) for file in files])

# Step 3: Run benchmarks and get mean runtime, in nanoseconds
run_bench_cmd = subprocess.run(
    ["cargo", "criterion", "--message-format=json", "search::federalist::liberty"],
    stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL,
    text=True
)

grep_for_success_cmd = subprocess.run(
    ["grep", "benchmark-complete"],
    input=run_bench_cmd.stdout,
    stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL,
    text=True
)


jq_cmd = subprocess.run(
    ["jq", ".mean.estimate"],
    input=grep_for_success_cmd.stdout,
    capture_output=True,
    text=True
)

bench_time_ns = float(jq_cmd.stdout)
bench_time_ms = bench_time_ns / 1000000

# Step 4: Print out results
sizes.update({
    "search::federalist::liberty": bench_time_ms
})

print(json.dumps(sizes, indent=2))
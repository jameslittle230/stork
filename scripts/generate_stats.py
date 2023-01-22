import os
import sys
import json
import subprocess

if not os.path.exists(os.path.join(os.getcwd(), ".stork-project-root")):
    print(
        f"Current working directory {os.getcwd()} doesn't look to be the Stork project root.\nRun this as `just upload` or run it from the Stork root directory. Exiting.")
    exit(1)

# Step 1: get file sizes for various distributed files
files = [
    'js/dist/stork.js',
    'js/dist/stork.wasm',
    'dev/indexes/federalist.st'
]

sizes = dict([(file.split('/')[-1], float(os.path.getsize(file))/1000)
              for file in files])

# Step 2: Run benchmarks and get mean runtime for each
benchmarks = [
    "build/federalist",
    "search/federalist/liberty"
]

for bench_name in benchmarks:
    print(f"Running benchmark for {bench_name}", file=sys.stderr)
    run_bench_cmd = subprocess.run(
        ["just", "bench", bench_name],
        stdout=subprocess.PIPE,
        text=True
    )

    # TODO: Restore benchmarks

    success_line = next((line for line in run_bench_cmd.stdout.splitlines() if "benchmark-complete" in line))

    success_line_dict = json.loads(success_line)

    bench_time_ms = round(float(success_line_dict['mean']['estimate']) / 1_000_000, 4)

    sizes.update({
        bench_name: bench_time_ms
    })

# Step 3: Print out results
print(json.dumps(sizes, indent=2))

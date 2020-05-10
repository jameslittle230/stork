import os
import humanize
import subprocess

files = [
    './dist/federalist.st',
    './dist/stork.wasm',
    './dist/stork.js'
]

for file in files:
    filesize_int = os.path.getsize(file)

    name_print = file.split('/')[-1]
    filesize_print = humanize.naturalsize(filesize_int, format='%.2f')

    print(f"{name_print}\t{filesize_print}")

times = []
for i in range(10):
    completed_process = subprocess.run(
        ["cargo", "run", "--", "--search", "./dist/federalist.st", "liber old world"], capture_output=True)
    time_string = completed_process.stderr.splitlines(
    )[-1].decode('utf-8').split(' ')[-2].split('s')[0]
    times.append(float(time_string))

time_print = sum(times) / len(times)
print(f"search duration (10-run mean)\t{time_print:.3f} seconds")

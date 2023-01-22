import os.path
import subprocess

configs = ["3b1b", "federalist", "bowdoin-orient"]

if not os.path.exists(os.path.join(os.getcwd(), ".stork-project-root")):
    print(
        f"Current working directory {os.getcwd()} doesn't look to be the Stork project root.\nRun this as `just upload` or run it from the Stork root directory. Exiting.")
    exit(1)

for config in configs:
    input  = os.path.join(os.getcwd(), "dev", "configs", f"{config}.toml")
    output = os.path.join(os.getcwd(), "dev", "indexes", f"{config}.st")

    if os.path.exists(output):
        print(f"dev/indexes/{config}.st already exists, skipping")
        continue
    
    if not os.path.exists(input):
        print(f"dev/configs/{config}.toml doesn't exist, skipping")

    subprocess.run(
        ["cargo", "run", "-q", "--all-features", "--", "build", "--input", input, "--output", output],
        stdout=subprocess.PIPE,
        text=True
    )
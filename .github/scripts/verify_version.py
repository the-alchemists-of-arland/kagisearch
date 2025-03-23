import toml
import os

cargoPath = 'Cargo.toml'

with open(cargoPath, 'r') as f:
    cargo_toml = toml.load(f)

version = os.environ.get("GIT_TAG_NAME")
if cargo_toml['package']['version'] != version:
    exit(1)

exit(0)

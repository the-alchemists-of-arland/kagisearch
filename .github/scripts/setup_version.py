import toml
import os

cargoPath = 'Cargo.toml'

with open(cargoPath, 'r') as f:
    cargo_toml = toml.load(f)

version = os.environ.get("GIT_TAG_NAME")
cargo_toml['package']['version'] = version

with open(cargoPath, 'w') as f:
    toml.dump(cargo_toml, f)

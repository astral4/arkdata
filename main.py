import tomllib
from pathlib import Path
import kawapack

if __name__ == "__main__":
    with open("config.toml", mode="rb") as f:
        config = tomllib.load(f)["unpack-settings"]

    input_dir = config["input_dir"]
    portrait_dir = Path(input_dir, "arts", "charportraits")

    kawapack.extract_all(
        input_dir,
        config["output_dir"],
        config.get("match_patterns"),
        portrait_dir=portrait_dir
    )
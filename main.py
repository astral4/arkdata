import tomllib
import kawapack

if __name__ == "__main__":
    with open("config.toml", mode="rb") as f:
        config = tomllib.load(f)["unpack-settings"]

    kawapack.convert(config["input_dir"], config["output_dir"])
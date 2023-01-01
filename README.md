<h1 align="center">welcome to scraping-icp-progects</h1>

This project maintains program code for scraping project information from the [ICP Projects page](https://n7ib3-4qaaa-aaaai-qagnq-cai.raw.ic0.app/) and outputting it to a json file.

When the program execution ends normally, a json file is output in the following format.

Output destination: `./data/dapps.json`
```json
{
  "55": {
    "logo_url": "https://pbs.twimg.com/profile_images/1470516138056454148/0wgyA2eE_400x400.jpg",
    "project_name": "DfinityJP",
    "data_social": [
      {
        "name": "@dfinityjpn",
        "url": "https://twitter.com/dfinityjpn"
      },
      {
        "name": "Discord",
        "url": "https://discord.gg/CR2vHZDqhF"
      },
      {
        "name": "Medium",
        "url": "https://medium.com/dfinityjp"
      }
    ],
    "category_list": [
      "Communities"
    ],
    "description": "DfinityJP is a non-profit organization for the promotion of Dfinity in Japan. DfinityJP was launched on 23/11/2021."
  },
  {...}
}
```

## 🚀 Usage

#### Installation

Run the following command to install Rust.
(Please refer to [here](https://www.rust-lang.org/ja/tools/install) for how to install)


```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

If you already have Rust installed, make sure you are up to date:

```bash
rustup update
```

Check tools version.(The execution result display differs depending on the environment.)

Rust
```bash
rustc --version
# rustc 1.65.0 (897e37553 2022-11-02)
```

Cargo (Cargo is the Rust package manager. When you install rust, cargo is also included)
```bash
cargo --version
# cargo 1.65.0 (4bc8f24d3 2022-10-20)
```

Clone this repository.
```bash
git clone https://github.com/iU-C3F/scraping-icp-projects.git
cd scraping-icp-projects
```

Run command.
```bash
cargo run src/main.rs
```

By editing the .env file, you can change the log level and whether or not to acquire screen captures.
```bash
# Specify whether to acquire screenshots.[true, false]
GET_SCREENSHOT = false
# Choice log level.[debug, error, info, warn]
RUST_LOG = "info"
# Specify output path of json file
DAPPS_JSON_PATH = "./data/dapps.json"
# Setting loading project elements.[true, false]
# true -> loading all project elements.
# false -> Scraping only the 64 projects that appear initially
LOADING_DATA = true
```

## Dependencies
```toml
[dependencies]
headless_chrome = "1.0.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dotenv = "0.15.0"
log = "0.4.0"
env_logger = "0.9.0"
```
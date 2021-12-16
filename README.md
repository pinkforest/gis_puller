gis_puller
==========

[![CI](https://github.com/pinkforest/gis_puller/actions/workflows/CI.yml/badge.svg)](https://github.com/pinkforest/gis_puller/actions/workflows/CI.yml)
[![Crates.io](https://img.shields.io/crates/v/gis_puller.svg)](https://crates.io/crates/gis_puller)
[![Docs](https://docs.rs/gis_puller/badge.svg)](https://docs.rs/gis_puller)
[![Deps](https://deps.rs/repo/github/pinkforest/gis_puller/status.svg)](https://deps.rs/repo/github/pinkforest/gis_puller)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![MSRV](https://img.shields.io/badge/MSRV-1.47.0-blue)

This Rust crate aims to abstract and implement a general global Geographic Information System (GIS) data catalog and acquisition functionality.

Acquired catalog will follow the data provider derived structural separations as-is.

To build on top the below crates can be in future used to consume the acquired data and beyond:

- gis_loader will be available to load/transform these acquired dataset(s) by this crate into various data stores
- gis_api will be available to provide gRPC API to consume the loaded dataset(s)

Everything should be considered unstable including the interface definitions.

## GIS Data Sources Supported

- Australia - Geoscape consumed via https://data.gov.au/

## GIS DataSets Catalogued

- Suburb/Locality Boundaries: Australia  (GDA94, GDA2020, GeoJSON)
- Address Forward/Reverse Lookup Data: Australia (GNAF)

Also see DIFFICULTIES.md

## Environment Variables

- USE_MODE = Environment "default" gets merged with either "development" | "integration" | "production" runtime configuration
- GIS_CONFIG_PATH = Filesystem path where the GIS datasource configuration files live
- TEST_DATA_PATH = Filesystem path where the mocks read data from including the "replay" recordings

## Usage

Using Rust Cargo - requires [cargo-edit](https://crates.io/crates/cargo-edit)

```bash
cargo add gis_puller
```

Or edit Cargo.toml by hand:

```toml
[dependencies]
gis_puller = "0.1.1"
```

Create documentation
```bash
env GIS_CONFIG_PATH=config RUN_MODE=development cargo doc --no-deps --open
```

Run tests (includes doctests and runs examples) with either replay or mocked datasources
```bash
env GIS_CONFIG_PATH=config TEST_DATA_PATH=test_data RUN_MODE=development cargo test
```

Run [clippy](https://github.com/rust-lang/rust-clippy) for additional lints
```bash
cargo clippy --all-targets --all-features
```

Run [tarpaulin](https://github.com/xd009642/tarpaulin) for code/examples test coverage
```bash
env GIS_CONFIG_PATH=config TEST_DATA_PATH=test_data RUN_MODE=development cargo +nightly tarpaulin --run-types Tests,Doctests,Benchmarks,Examples,Lib,Bins -v
```
NOTE: __Tarpaulin requires +nightly: rustup toolchain install nightly__

@TODO binary crate -- To use in shell with gis_puller image Docker e.g. with Australia localities boundaries datasets obtained from data.gov.au:
```bash
docker run --rm -t -i -e env GIS_PULLER_CONFIGPATH=/gis/etc/gis_puller GIS_PULLER_DATAPATH=/gis/data gis_puller gis_puller pull --all au locality/boundary
```

Or to use as a library:
- TBD: [Interfaces](INTERFACES.md) to appear in future gis_puller::Puller

Currently with AU locality/boundary dataset I doctest fetcher_matcher the below in development the below with recorded result replayed as a mock:

```rust
mod mocks;
use mocks::http_server;
use httpmock::Method::GET;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let au_config = task::spawn_blocking(move || {
    gis_puller::settings::Settings::load("au")
  }).await?.unwrap();

  // @TODO: replace this mess with a macro_rules!
  let mut au_fetcher_matcher = (*au_config).fetcher_matcher.clone();

  let mock_server = mocks::http_server::serve().await?;
  let mock_path   = format!("/replay/data.gov.au/api/v0/search/datasets");
  let mock_query  = format!("localities-boundaries");

  let inject_replay = format!("test_data/replay/au/locality_boundaries.json");
  let inject_url = format!("http://localhost:{}{}", mock_server.port(), &mock_path);
  
  au_fetcher_matcher.rest_url = Some(inject_url);
  au_fetcher_matcher.query = Some(mock_query.clone());

  let mock = mock_server.mock(|when, then| {
    when.method(GET)       
      .path(&mock_path)
      .query_param("query", &mock_query);
    then.status(200)
      .header("Content-Type", "application/json")
      .body_from_file(&inject_replay);
  });

  let catalog = gis_puller::au::fetcher_matcher(&au_fetcher_matcher).await?;

  println!("Catalog = {:#?}", catalog);

  Ok(())
}  
```

Scratchpad for future interface:
```ignore

let puller_au = gis_puller::Puller::load('au').await?;

let sources_au = puller_au.sources('locality/boundary').await?;

// make sources_au Iterable for .iter()

sources_au.pull_all('/tmp').await?;
sources_au.pull_one('nsw').await?;

```

[cargo-edit]: https://crates.io/crates/cargo-edit

The below features can be toggled via Cargo:

- 'xxx': Enables xxx

## License

Licensed under either of:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

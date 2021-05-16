Interfaces
==========

Interfaces should appear when we add more than the AU data source when we have more general idea how to abstract multiple data sources with multiple sources available.

@TODO Worth it? Async-trait to provide abstracted interface to submods
@TODO Worth it? https://internals.rust-lang.org/t/async-traits-the-less-dynamic-allocations-edition/13048/2                                                                                           

## Interface future?

```rust
let puller_au = gis_puller::Puller::new('au');
let sources_au = gis_puller::Catalog::sources(settings_au, 'localities/boundaries').await?;
sources_au.pull_all('/tmp').await?;
sources_au.pull_all_tmp().await?;
sources_au.pull_one('nsw').await?;
```

## Errors?

PullerError 1st class enum:
 - rest/API HTTP/protocol errors
 - JSON/data deserialsiation/corruption etc. errors - enforce schemas?
 - 
# joat-path-rs

[![CI](https://github.com/rcook/joat-path-rs/actions/workflows/ci.yaml/badge.svg)][ci-workflow]
[![Publish](https://github.com/rcook/joat-path-rs/actions/workflows/publish.yaml/badge.svg)][publish-workflow]
[![crates.io](https://img.shields.io/crates/v/joat-path.svg)][crates-io]
[![Docs](https://docs.rs/joat-path/badge.svg)](https://docs.rs/joat-path)

Absolute paths

## Use package

Get it from [crates.io][crates-io]:

```bash
cargo add joat-path
```

## About

This is a fork of [path-clean][path-clean]. The main distinguishing feature is that this
package provides the following three APIs:

* `clean`: clean paths according to rules of host operating system (i.e. Unix on Unix, Windows on Windows)
* `clean_unix`: clean paths according to Unix rules
* `clean_windows`: clean paths according to Windows rules

This enables manipulation of Unix paths on Windows and Windows paths on Unix which has
real-world applications. TBD: Document the real-world applications here.


[ci-workflow]: https://github.com/rcook/joat-path-rs/actions/workflows/ci.yaml
[crates-io]: https://crates.io/crates/joat-path
[path-clean]: https://github.com/danreeves/path-clean
[publish-workflow]: https://github.com/rcook/joat-path-rs/actions/workflows/publish.yaml

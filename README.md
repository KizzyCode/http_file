[![License](https://img.shields.io/badge/License-BSD%202--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)

# About
This library provides POSIX-like random-read-access to a HTTP-resource

It provides the following features:
 - `open`ing a HTTP-resource: this validates if the server supports HTTP-range-requests (required for random access), gets
   the resources's size and removes the percent-encoding to display a human-readable filename
 - `read`/`read_at`, `seek`/`tell` and some helper-APIs

# Dependencies
This library depends on [network_io](https://github.com/KizzyCode/network_io) for the network-operations and
[http](https://github.com/KizzyCode/http) for the HTTP-encoding/decoding.

# Build and installation
To build the documentation, go into the projects root-directory and run `cargo doc --release`; to open the documentation
in your web-browser, run `cargo doc --open`.

To build the library, go into the projects root-directory and run `cargo build --release`; you can find the build in
target/release.

# TODO:
 - HTTPS-support
 - Background-prefetching for better performance
 - Maybe adaptive cache-resizing
[![License](https://img.shields.io/badge/License-BSD%202--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)

# About
This library provides POSIX-like random-read-access to a HTTP-URI

We provide the following features:
 - "Opening" a HTTP-URI: this validates if the server supports HTTP-range-requests (required for random access), gets
   the file's size and removes the percent-encoding to display a human-readable filename
 - `read`/`read_at`, `seek`/`tell` and some helper-APIs
 
# Build and installation
To build the documentation, go into the projects root-directory and run `cargo doc --release`; to open the documentation
in your web-browser, run `cargo doc --open`.

To build the library, go into the projects root-directory and `run cargo build --release`; you can find the build in
target/release.

# TODO:
 - HTTPS-support
 - Background-prefetching for better performance
 - Maybe adaptive cache-resizing
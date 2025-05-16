# Get Up and Running

## On MacOS

I assume that [homebrew](https://brew.sh) and Xtool command-line utilities are already installed.

### Install rust toolchain

The rust toolchain is installed via Homebrew as follows:

* > % `brew install rustup-init`
* > % `rustup-init`

Then, follow instructions (default values are recommended).
After that, check by running the following command:

* > % `cargo version` \
    `cargo 1.59.0 (49d8809dc 2022-02-10)`

###  Build and Run the Project

I assume that the current directory is the same as this file.

* To build the debug version: \
  > `cargo build`
* To build the release (optimized) version: \
  > `cargo build --release`

To run via cargo:
* > % `cargo run --bin synth-lights -- full 2`
or
* > % `cargo run --bin synth-lights --release -- full 2`

To run the executable directly:
* > % `./target/release/synth-lights full 2`

To output the results to a file:
* > % `./target/release/synth-lights full 2 | tee output.txt`

###  Test the Project

If there are errors, it is a good idea to run the tests, as follows:
* > % `cargo test`

### Check Dependencies

It is possible to check vulnerabilities or other problems that may occur in the dependencies. This requires to install an extension to cargo as follows:

* > % `cargo install cargo-deny`

After `cargo-deny` has been added, the dependencies can be checked as follows:

* > % `cargo deny check`

### Generate API Documentations

The documentation is very sparse, but can nevertheless be generated via the following command:
* > % `cargo doc --no-deps`
or remove the `--no-deps` and it will generate the doc for all dependencies (many files).

Then, the documentation can be accessed from the following file [`target/doc/synth_lights/index.html`](target/doc/synth_lights/).

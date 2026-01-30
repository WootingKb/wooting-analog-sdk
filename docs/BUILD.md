## Building

### Build Dependencies

- [rust](https://www.rust-lang.org/)
- [cbindgen](https://github.com/eqrion/cbindgen) (For verifying/generating headers. Should be
  installed automatically if necessary)
- [wixtoolset](https://wixtoolset.org/releases/) If you want to build the windows installer for the
  sdk **[Windows]**

### How to Build

Everything can be built using this command. All the outputs will be under `target/debug`

```bash
# Normal debug build without any extra features
cargo build

# Builds the library with all the exported ffi symbols
cargo build --features ffi

# Builds the library with support for the virtual keyboard to hook into the SDK
cargo build --features virtual-input
```

The current build process is setup to verify the existing generated headers in the test phase. If
you decide to make changes which effect these outputs, you can update the headers by running:

```bash
cargo install cbindgen@0.29.2

# Rebuild headers
cbindgen --crate wooting-analog-sdk --output ./includes/wooting-analog-sdk.h

# Verify
cbindgen --crate wooting-analog-sdk --output ./includes/wooting-analog-sdk.h --verify
```

To run the virtual keyboard (The Analog SDK must be running for this to work):

```bash
cargo run -p wooting-analog-virtual-control
```

To build the windows installer for the SDK:

```bash
cargo install cargo-wix@0.3.9
cargo wix -p wooting-analog-sdk --nocapture
```

The installer will be located in `$gitroot/target/wix`

To build the deb package for the SDK:

```bash
cargo install cargo-deb@3.6.2
cargo deb
```

The deb package will be located in `$gitroot/target/debian`

### Outputs

All build outputs can be found under `target/debug`, with generated headers coming under the
`includes` directory.

Currently the headers have to be manually generated and kept in the repo. When intentional changes
are made, the testing phase verifies that the pre-generated headers match what would be generated
now to ensure that accidental changes aren't made to the output of the header generation.
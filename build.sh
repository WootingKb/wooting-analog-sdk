#/bin/sh
#cargo build --manifest-path analog-sdk/Cargo.toml
#cargo build --manifest-path analog-sdk-wrapper/Cargo.toml
#cp analog-sdk-wrapper/target/debug/libanalog_sdk_wrapper.so analog-sdk-test
cargo build
./plugin.sh

RUST_LOG=trace dotnet run --project analog-sdk-test
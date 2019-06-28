#/bin/sh
cargo build --manifest-path analog-sdk/Cargo.toml
cargo build --manifest-path analog-sdk-wrapper/Cargo.toml
#cp analog-sdk-wrapper/target/debug/libanalog_sdk_wrapper.so analog-sdk-test
./plugin.sh

cd analog-sdk-test
dotnet run
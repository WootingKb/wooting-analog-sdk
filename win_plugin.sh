#/bin/sh
cargo build --manifest-path analog-plugin/Cargo.toml --target x86_64-pc-windows-gnu
cd analog-plugin-c
make cross
cd ..
cp analog-plugin/target/x86_64-pc-windows-gnu/debug/analog_plugin.dll analog-plugins
#cp analog-plugin-c/cplugin.dll analog-plugins

#cd analog-sdk
#cargo run
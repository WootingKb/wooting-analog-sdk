#/bin/sh
cargo build --manifest-path analog-plugin/Cargo.toml
cd analog-plugin-c
make
cd ..
cp analog-plugin/target/debug/libanalog_plugin.so analog-plugins
#cp analog-plugin-c/cplugin.so analog-plugins

#cd analog-sdk
#cargo run
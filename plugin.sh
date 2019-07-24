#/bin/sh
cargo build --manifest-path analog-plugin/Cargo.toml
cmake --build analog-plugin-c/out --target analog_plugin_c -- -j 4
#cmake -S analog-plugin-c -B analog-plugin-c/out
#make -s -C analog-plugin-c/out

cp analog-plugin/target/debug/libanalog_plugin.so analog-plugins
cp analog-plugin-c/out/libanalog_plugin_c.so analog-plugins

#cd analog-sdk
#cargo run
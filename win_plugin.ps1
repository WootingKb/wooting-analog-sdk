cargo build --manifest-path analog-plugin/Cargo.toml
#cd analog-plugin-c
#make cross
#cd ..
rm analog-plugins\analog_plugin.dll
move analog-plugin\target\debug\analog_plugin.dll analog-plugins
#cp analog-plugin-c/cplugin.dll analog-plugins
#cd analog-sdk
#cargo run
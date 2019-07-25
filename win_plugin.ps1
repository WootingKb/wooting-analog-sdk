#cargo build --manifest-path analog-plugin/Cargo.toml
cmake.exe -S analog-plugin-c -B analog-plugin-c\win-out
MSBuild.exe .\analog-plugin-c\win-out\analog_plugin_c.sln
#cd analog-plugin-c
#make cross
#cd ..
rm analog-plugins\analog_plugin.dll
move target\debug\analog_plugin.dll analog-plugins

rm analog-plugins\analog_plugin_c.dll
move analog-plugin-c\win-out\Debug\analog_plugin_c.dll analog-plugins
#cp analog-plugin-c/cplugin.dll analog-plugins
#cd analog-sdk
#cargo run
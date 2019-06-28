#/bin/sh
cargo build --manifest-path analog-sdk/Cargo.toml --target x86_64-pc-windows-gnu
cargo build --manifest-path analog-sdk-wrapper/Cargo.toml --target x86_64-pc-windows-gnu
#fix naming of wrapper dll
rm analog-sdk-wrapper/target/x86_64-pc-windows-gnu/debug/libanalog_sdk_wrapper.dll
mv analog-sdk-wrapper/target/x86_64-pc-windows-gnu/debug/analog_sdk_wrapper.dll analog-sdk-wrapper/target/x86_64-pc-windows-gnu/debug/libanalog_sdk_wrapper.dll

#fix naming of sdk dll
rm analog-sdk/target/x86_64-pc-windows-gnu/debug/libanalog_sdk.dll
mv analog-sdk/target/x86_64-pc-windows-gnu/debug/analog_sdk.dll analog-sdk/target/x86_64-pc-windows-gnu/debug/libanalog_sdk.dll

#Copy to the test directory if needed
#cp analog-sdk-wrapper/target/x86_64-pc-windows-gnu/debug/libanalog_sdk_wrapper.dll analog-sdk-test
#cp analog-sdk/target/x86_64-pc-windows-gnu/debug/libanalog_sdk.dll analog-sdk-test
./win_plugin.sh

#cd analog-sdk-test
#dotnet run
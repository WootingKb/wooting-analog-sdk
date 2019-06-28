cargo build --manifest-path .\analog-sdk\Cargo.toml
cargo build --manifest-path .\analog-sdk-wrapper\Cargo.toml
#fix naming of wrapper dll
rm analog-sdk-wrapper\target\debug\libanalog_sdk_wrapper.dll
move analog-sdk-wrapper\target\debug\analog_sdk_wrapper.dll analog-sdk-wrapper\target\debug\libanalog_sdk_wrapper.dll

#fix naming of sdk dll
rm analog-sdk\target\debug\libanalog_sdk.dll
move analog-sdk\target\debug\analog_sdk.dll analog-sdk\target\debug\libanalog_sdk.dll

#Copy to the test directory if needed
#xcopy analog-sdk-wrapper\target\debug\libanalog_sdk_wrapper.dll analog-sdk-test
#xcopy analog-sdk\target\debug\libanalog_sdk.dll analog-sdk-test
.\win_plugin.ps1

cd analog-sdk-test
dotnet run
cd ..

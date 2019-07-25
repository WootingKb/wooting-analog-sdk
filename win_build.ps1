#cargo build --manifest-path .\analog-sdk\Cargo.toml
#cargo build --manifest-path .\analog-sdk-wrapper\Cargo.toml
Remove-Item Env:RUST_LOG
cargo build
#fix naming of wrapper dll
rm target\debug\libanalog_sdk_wrapper.dll
move target\debug\analog_sdk_wrapper.dll target\debug\libanalog_sdk_wrapper.dll

#fix naming of sdk dll
rm target\debug\libanalog_sdk.dll
move target\debug\analog_sdk.dll target\debug\libanalog_sdk.dll

#Copy to the test directory if needed
#xcopy analog-sdk-wrapper\target\debug\libanalog_sdk_wrapper.dll analog-sdk-test
#xcopy analog-sdk\target\debug\libanalog_sdk.dll analog-sdk-test
.\win_plugin.ps1
Set-Item -Path Env:RUST_LOG -Value ("trace")
dotnet run --project .\analog-sdk-test
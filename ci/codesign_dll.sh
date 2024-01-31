# Thanks https://github.com/electron-userland/electron-builder/issues/3629#issuecomment-473238513
if [ $RUNNER_OS = Windows ]; then
  set -e


  # curl -v -L "$WIN_CSC_LINK" --output cert.pfx

  # powershell Set-ExecutionPolicy -ExecutionPolicy Unrestricted -Scope LocalMachine
  # powershell Get-ExecutionPolicy -List

  ROOT_DIR=${GITHUB_WORKSPACE:-.}
  ARTIFACT_FOLDER=$ROOT_DIR/target/release-artifacts

  powershell $ROOT_DIR/ci/codesign.ps1 $ARTIFACT_FOLDER/wooting_analog_sdk.dll $ARTIFACT_FOLDER/wooting_analog_plugin.dll $ARTIFACT_FOLDER/wooting_analog_wrapper.dll $ARTIFACT_FOLDER/wooting-analog-sdk-updater.exe $ARTIFACT_FOLDER/wooting_analog_test_plugin.dll $ARTIFACT_FOLDER/wooting-analog-virtual-control.exe

  # powershell $ROOT_DIR/ci/codesign.ps1 $ARTIFACT_FOLDER/wooting_analog_sdk.dll
  # powershell $ROOT_DIR/ci/codesign.ps1 $ARTIFACT_FOLDER/wooting_analog_plugin.dll
  # powershell $ROOT_DIR/ci/codesign.ps1 $ARTIFACT_FOLDER/wooting_analog_wrapper.dll
  # powershell $ROOT_DIR/ci/codesign.ps1 $ARTIFACT_FOLDER/wooting-analog-sdk-updater.exe

  # powershell $ROOT_DIR/ci/codesign.ps1 $ARTIFACT_FOLDER/wooting_analog_test_plugin.dll
  # powershell $ROOT_DIR/ci/codesign.ps1 $ARTIFACT_FOLDER/wooting-analog-virtual-control.exe
fi
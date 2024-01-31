# Thanks https://github.com/electron-userland/electron-builder/issues/3629#issuecomment-473238513
if [ $RUNNER_OS = Windows ]; then
  set -e

  export PATH="C:\Program Files (x86)\Windows Kits\10\bin\x64":$PATH
  # TODO: Dynamic installer filename
  #export BINARY_FILE="target/wix/wooting_analog_sdk-0.1.0-x86_64.msi"

#  choco install -y windows-sdk-10.0

  # curl -v -L "$WIN_CSC_LINK" --output cert.pfx

  powershell Set-ExecutionPolicy -ExecutionPolicy Unrestricted -Scope LocalMachine
  powershell Get-ExecutionPolicy -List

  powershell $GITHUB_WORKSPACE/ci/codesign.ps1 $WIN_INSTALLER_PATH
fi
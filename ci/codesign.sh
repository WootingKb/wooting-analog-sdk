if [ $TRAVIS_OS_NAME = windows ]; then
  set -e

  export PATH="C:\Program Files (x86)\Windows Kits\10\bin\x64":$PATH
  # TODO: Dynamic installer filename
  export BINARY_FILE="target/wix/wooting_analog_sdk-0.1.0-x86_64.msi"

  choco install -y windows-sdk-10.0

  curl -v -L "$WIN_CSC_LINK" --output cert.pfx
  # gpg --passphrase ${WIN_CSC_KEY_PASSWORD} --batch -o cert.pfx -d cert.pfx.gpg

  powershell Set-ExecutionPolicy -ExecutionPolicy Unrestricted -Scope LocalMachine
  powershell Get-ExecutionPolicy -List

  powershell $PWD/ci/codesign.ps1
  signtool.exe verify -pa "$BINARY_FILE"
fi
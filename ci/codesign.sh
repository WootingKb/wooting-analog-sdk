if [ $TRAVIS_OS_NAME = windows ]; then
  set -e

  export PATH="C:\Program Files (x86)\Windows Kits\10\bin\x64":$PATH
  export BINARY_FILE=$1

  # choco install -y windows-sdk-10.0

  curl -v -L "$WIN_CSC_LINK" --output cert.pfx
  # gpg --passphrase ${WIN_CSC_KEY_PASSWORD} --batch -o cert.pfx -d cert.pfx.gpg

  powershell Set-ExecutionPolicy -ExecutionPolicy Unrestricted -Scope LocalMachine
  powershell Get-ExecutionPolicy -List

  powershell $PWD/.build/codesign.ps1
  signtool.exe verify -pa "$BINARY_FILE"
fi
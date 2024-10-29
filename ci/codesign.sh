# Thanks https://github.com/electron-userland/electron-builder/issues/3629#issuecomment-473238513
if [ $RUNNER_OS = Windows ]; then
  set -e


  powershell Set-ExecutionPolicy -ExecutionPolicy Unrestricted -Scope LocalMachine
  powershell Get-ExecutionPolicy -List

  powershell $GITHUB_WORKSPACE/ci/codesign.ps1 $WIN_INSTALLER_PATH
fi

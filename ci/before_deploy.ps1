# This script takes care of packaging the build artifacts that will go in the
# release zipfile

$SRC_DIR = $PWD.Path
$STAGE = [System.Guid]::NewGuid().ToString()

Set-Location $ENV:Temp
New-Item -Type Directory -Name $STAGE
Set-Location $STAGE

New-Item -Type Directory -Name "plugins"
New-Item -Type Directory -Name "plugins\lib"
New-Item -Type Directory -Name "plugins\includes"
New-Item -Type Directory -Name "plugins\includes-cpp"

New-Item -Type Directory -Name "wrapper"
New-Item -Type Directory -Name "wrapper\includes"
New-Item -Type Directory -Name "wrapper\includes-cpp"
New-Item -Type Directory -Name "wrapper\sdk"


$ZIP = "$SRC_DIR\$($Env:CRATE_NAME)-$($Env:APPVEYOR_REPO_TAG_NAME)-$($Env:TARGET).zip"

# Copy Plugin items
Copy-Item "$SRC_DIR\target\$($Env:TARGET)\release\wooting_analog_common.lib" '.\plugins\lib'

## Copy c headers
Copy-Item "$SRC_DIR\includes\plugin.h" '.\plugins\includes\'
Copy-Item "$SRC_DIR\includes\wooting-analog-plugin-dev.h" '.\plugins\includes\'
Copy-Item "$SRC_DIR\includes\wooting-analog-common.h" '.\plugins\includes\'

## Copy cpp headers
Copy-Item "$SRC_DIR\includes-cpp\wooting-analog-plugin-dev.h" '.\plugins\includes-cpp\'
Copy-Item "$SRC_DIR\includes-cpp\wooting-analog-common.h" '.\plugins\includes-cpp\'

## Copy docs
Copy-Item "$SRC_DIR\PLUGINS.md" '.\plugins\'



# Copy wrapper items
Copy-Item "$SRC_DIR\target\$($Env:TARGET)\release\wooting_analog_wrapper.dll" '.\wrapper\'
Copy-Item "$SRC_DIR\target\$($Env:TARGET)\release\wooting_analog_sdk.dll" '.\wrapper\sdk\'

## Copy c headers
Copy-Item "$SRC_DIR\includes\wooting-analog-wrapper.h" '.\wrapper\includes\'
Copy-Item "$SRC_DIR\includes\wooting-analog-common.h" '.\wrapper\includes\'

## Copy cpp headers
Copy-Item "$SRC_DIR\includes-cpp\wooting-analog-wrapper.h" '.\wrapper\includes-cpp\'
Copy-Item "$SRC_DIR\includes-cpp\wooting-analog-common.h" '.\wrapper\includes-cpp\'

## Copy docs
Copy-Item "$SRC_DIR\SDK_USAGE.md" '.\wrapper\'

# Copy README
#Copy-Item "$SRC_DIR\README.md" '.\wrapper\'

7z a "$ZIP" *

Push-AppveyorArtifact "$ZIP"

Remove-Item *.* -Force
Set-Location ..
Remove-Item $STAGE
Set-Location $SRC_DIR

# Thanks https://github.com/electron-userland/electron-builder/issues/3629#issuecomment-473238513
# Set-PSDebug -Trace 1
$ErrorActionPreference = "Stop"
# dir cert:/LocalMachine

# $WINDOWS_SDK_VER = '10.0.17763.0'
$WINDOWS_SDK_VER = '10.0.22000.0'

# Remember what the Path was before so we can clean it up after exiting
$PREV_PATH = $env:PATH

$env:PATH += ";C:/Program Files (x86)/Windows Kits/10/bin/$WINDOWS_SDK_VER/x64/"

# Passing in $args allows the caller to specify multiple files to be signed at once
signtool.exe sign /fd sha256 /td sha256 /tr ${Env:TIMESTAMP}?td=sha256 /f $Env:CERT_FILE /csp "$Env:CRYPT_PROVIDER" /kc "[${Env:READER}{{${Env:PASS}}}]=${Env:CONTAINER}" $args
signtool.exe verify /pa $args

$env:PATH = $PREV_PATH

# Thanks https://github.com/electron-userland/electron-builder/issues/3629#issuecomment-473238513
# Set-PSDebug -Trace 1
$ErrorActionPreference = "Stop"
# dir cert:/LocalMachine

# $WINDOWS_SDK_VER = '10.0.17763.0'
$WINDOWS_SDK_VER = '10.0.22000.0'

# Remember what the Path was before so we can clean it up after exiting
$PREV_PATH = $env:PATH

$env:PATH += ";C:/Program Files (x86)/Windows Kits/10/bin/$WINDOWS_SDK_VER/x64/"

# $Password = ConvertTo-SecureString -String $Env:WIN_CSC_KEY_PASSWORD -AsPlainText -Force
# Import-PfxCertificate -FilePath cert.pfx -CertStoreLocation Cert:\LocalMachine\My -Password $Password

# Passing in $args allows the caller to specify multiple files to be signed at once
signtool.exe sign /tr $env:TimestampServer /td sha256 /fd sha256 /n $Env:WIN_CSC_SUBJECTNAME $args
signtool.exe verify /pa $args
# Start-Process -NoNewWindow -Wait 'signtool.exe' -ArgumentList "sign /tr `"$env:TimestampServer`" /td sha256 /fd sha256 /n `"$Env:WIN_CSC_SUBJECTNAME`" `"$File`""
# Start-Process -NoNewWindow -Wait 'signtool.exe' -ArgumentList "verify /pa `"$File`""

$env:PATH = $PREV_PATH
# Thanks https://github.com/electron-userland/electron-builder/issues/3629#issuecomment-473238513
Set-PSDebug -Trace 1

dir cert:/LocalMachine

$Password = ConvertTo-SecureString -String $Env:WIN_CSC_KEY_PASSWORD -AsPlainText -Force
Import-PfxCertificate -FilePath cert.pfx -CertStoreLocation Cert:\LocalMachine\My -Password $Password
Start-Process -NoNewWindow -Wait 'C:/Program Files (x86)/Windows Kits/10/bin/10.0.17763.0/x86/signtool.exe' -ArgumentList "sign -v -sm -s My -n `"$Env:WIN_CSC_SUBJECTNAME`" -d `"$Env:WIN_CSC_DESC`" `"$Env:WIN_INSTALLER_PATH`""
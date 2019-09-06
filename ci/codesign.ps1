Set-PSDebug -Trace 1

dir cert:/LocalMachine

$Password = ConvertTo-SecureString -String $Env:WIN_CSC_KEY_PASSWORD -AsPlainText -Force
Import-PfxCertificate -FilePath cert.pfx -CertStoreLocation Cert:\LocalMachine\My -Password $Password
Start-Process -NoNewWindow -Wait signtool.exe -ArgumentList "sign -v -sm -s My -n `"$Env:WIN_CSC_SUBJECTNAME`" -d `"$Env:WIN_CSC_DESC`" `"$Env:BINARY_FILE`""
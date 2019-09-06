Set-PSDebug -Trace 1

dir cert:/LocalMachine

$Password = ConvertTo-SecureString -String $Env:WIN_CSC_KEY_PASSWORD -AsPlainText -Force
Import-PfxCertificate -FilePath cert.pfx -CertStoreLocation Cert:\LocalMachine\My -Password $Password
Start-Process -PassThru -Wait signtool.exe -ArgumentList "sign -v -sm -s My -n `"$Env:CERT_SUBJECTNAME`" -d `"$Env:CODESIGN_DESC`" `"$Env:BINARY_FILE`""
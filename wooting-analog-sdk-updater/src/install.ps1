param([string]$msi_path="")
echo "Waiting 2 seconds before starting to ensure updater exe has closed"
Start-Sleep -Seconds 2
echo "Executing installer"
Start-Process -FilePath "msiexec" -ArgumentList "/i $msi_path /quiet /qb /norestart" -Wait

# Cleanup tmp dir, thanks: https://stackoverflow.com/questions/42439961/delete-the-self-folder-once-execution-completed
function Delete() {
    $Invocation = (Get-Variable MyInvocation -Scope 1).Value
    $Path =  $Invocation.MyCommand.Path
    Write-Host $Path
    Remove-Item (Split-Path $Path) -recurse -force
            }
echo "Cleaning up"
Delete
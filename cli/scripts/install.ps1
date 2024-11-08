# Can be either aarch64, i686, x86_64
function Get-Architecture {
  $Architecture = (Get-CimInstance Win32_Processor).Architecture
  if ($Architecture -eq 9) {
    return "aarch64"
  }
  elseif ($Architecture -eq 0) {
    return "i686"
  }
  elseif ($Architecture -eq 5) {
    return "x86_64"
  }
}

function Install {
  param(
    [string]$Version
  );

  if ($Version -match "^\d+\.\d+\.\d+$") {
    $Version = "v$Version"
  }
  elseif ($Version -match "^v\d+\.\d+\.\d+$") {
    $Version = "$Version"
  }

  $ROAN_DIR = "${Home}\.roan"

  if (-not (Test-Path $ROAN_DIR)) {
    New-Item -ItemType Directory -Path $ROAN_DIR
  }

  $BIN_DIR = mkdir -Force "${ROAN_DIR}\bin"

  $Arch = Get-Architecture;
  $File_Name = "${Arch}-pc-windows-msvc.zip"

    try {
    Remove-Item "${BIN_DIR}\roan.exe" -Force
  } catch [System.Management.Automation.ItemNotFoundException] {
    # ignore
  } catch [System.UnauthorizedAccessException] {
    $openProcesses = Get-Process -Name roan | Where-Object { $_.Path -eq "${BIN_DIR}\roan.exe" }
    if ($openProcesses.Count -gt 0) {
      Write-Output "Install Failed - An older installation exists and is open. Please close open Roan processes and try again."
      return 1
    }
    Write-Output "Install Failed - An unknown error occurred while trying to remove the existing installation"
    Write-Output $_
    return 1
  } catch {
    Write-Output "Install Failed - An unknown error occurred while trying to remove the existing installation"
    Write-Output $_
    return 1
  }
}

Install $args[0]
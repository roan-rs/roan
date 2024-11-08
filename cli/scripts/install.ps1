#!/usr/bin/env pwsh
param (
    [String]$Version = "latest"
)

# Can be either aarch64, i686, x86_64
function Get-ProcessorArchitecture
{
    $architecture = (Get-WmiObject -Class Win32_Processor).Architecture
    switch ($architecture)
    {
        0 {
            return "x86"
        }       # IA-32 (Intel 32-bit)
        5 {
            return "arm"
        }       # ARM
        6 {
            return "ia64"
        }      # Intel Itanium-based
        9 {
            return "x64"
        }       # x64 (Intel 64-bit)
        12 {
            return "aarch64"
        }  # ARM64
        default {
            return "unknown"
        }
    }
}

function Get-DetectedArchitecture
{
    switch (Get-ProcessorArchitecture)
    {
        "x86" {
            return "i686"
        }
        "x64" {
            return "x86_64"
        }
        "aarch64" {
            return "aarch64"
        }
        default {
            return "unknown"
        }
    }
}

function Colorize
{
    param (
        [string]$Color,
        [string]$Text
    )
    return "$( [char]27 + $Color )$Text$( [char]27 + '[0m' )"
}

function Add-BinDirToPath
{
    param (
        [string]$BinDir
    )

    if ($env:Path -notlike "*$BinDir*")
    {
        try
        {
            # Temporarily add to the current session's PATH
            $env:Path = "$BinDir;$env:Path"

            # Permanently add to the user's PATH
            [Environment]::SetEnvironmentVariable("Path", "$BinDir;$([Environment]::GetEnvironmentVariable("Path", "User") )", "User")
            Write-Output "🛠️ Added $BinDir to PATH"
        }
        catch
        {
            Write-Output "Warning: Failed to add $BinDir to PATH permanently. Please add it manually."
        }
    }
}

function Install-Roan
{
    param (
        [string]$Version
    )

    # Format version
    if ($Version -match "^\d+\.\d+\.\d+$")
    {
        $Version = "v$Version"
    }
    elseif (-not ($Version -match "^v\d+\.\d+\.\d+$"))
    {
        $Version = "latest"
    }

    # Paths and Architecture Setup
    $RoanDir = "$HOME\.roan"
    $BinDir = "$RoanDir\bin"
    if (-not (Test-Path $RoanDir))
    {
        New-Item -ItemType Directory -Path $RoanDir | Out-Null
    }
    if (-not (Test-Path $BinDir))
    {
        New-Item -ItemType Directory -Path $BinDir | Out-Null
    }

    $Arch = Get-DetectedArchitecture
    $FileName = "$Arch-pc-windows-msvc.zip"
    $DownloadPath = "$RoanDir\$FileName"

    # Remove existing executable
    try
    {
        Remove-Item "$BinDir\roan.exe" -Force -ErrorAction SilentlyContinue
    }
    catch [System.UnauthorizedAccessException]
    {
        $openProcesses = Get-Process -Name roan -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq "$BinDir\roan.exe" }
        if ($openProcesses)
        {
            Write-Output "Install Failed - Close open Roan processes and try again."
            return 1
        }
    }
    catch
    {
        Write-Output "Install Failed - Unable to remove the existing installation."
        Write-Output $_
        return 1
    }

    # Download and install
    $ArchDisplay = Colorize "[95m" $Arch
    $VersionDisplay = Colorize "[95m" $Version
    Write-Host "📦 Downloading Roan $VersionDisplay for $ArchDisplay..."

    $Url = "https://github.com/roan-rs/roan/releases/$(
    if ($Version -eq "latest")
    {
        "latest/download"
    }
    else
    {
        "download/$Version"
    } )/$FileName"

    # Attempt to download
    $downloaded = $false
    try
    {
        Invoke-WebRequest -Uri $Url -OutFile $DownloadPath -ErrorAction Stop
        $downloaded = $true
    }
    catch
    {
        Write-Warning "First download attempt failed. Retrying with Invoke-RestMethod..."
        try
        {
            Invoke-RestMethod -Uri $Url -OutFile $DownloadPath -ErrorAction Stop
            $downloaded = $true
        }
        catch
        {
            Write-Output "Install Failed - Could not download from $Url"
            return 1
        }
    }

    # Extract and setup binary
    if ($downloaded)
    {
        try
        {
            Expand-Archive -Path $DownloadPath -DestinationPath $BinDir -Force
            Write-Output "📚 Unzipped executable to $( Colorize '[95m' $BinDir )"
        }
        catch
        {
            Write-Output "Install Failed - Unable to unzip $DownloadPath"
            Write-Output $_
            return 1
        }

        # Rename the executable
        try
        {
            Rename-Item "$BinDir\roan-cli-$Arch-pc-windows-msvc.exe" "$BinDir\roan.exe" -Force
        }
        catch
        {
            Write-Output "Install Failed - Unable to rename the executable"
            return 1
        }

        # Clean up
        Remove-Item $DownloadPath -Force
        Write-Output "🎉 Successfully installed Roan $VersionDisplay for $ArchDisplay"

        # Add to PATH
        Add-BinDirToPath -BinDir $BinDir
    }
}

Install-Roan -Version $Version

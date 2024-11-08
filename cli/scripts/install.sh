#!/usr/bin/env bash

VERSION=${1:-"latest"}

# Determine the architecture
get_processor_architecture() {
    case "$(uname -m)" in
        "x86_64")
            echo "x86_64"
            ;;
        "i686" | "i386")
            echo "i686"
            ;;
        "aarch64")
            echo "aarch64"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# Determine the operating system and choose the correct artifact format
get_os_artifact_suffix() {
    case "$(uname -s)" in
        "Linux")
            echo "unknown-linux-gnu"
            ;;
        "Darwin")
            echo "apple-darwin"
            ;;
        "MINGW"* | "MSYS"* | "CYGWIN"*)
            echo "pc-windows-msvc"
            ;;
        *)
            echo "unknown-os"
            ;;
    esac
}

# Add directory to PATH if not already added
add_bin_dir_to_path() {
    local bin_dir="$1"
    if [[ ":$PATH:" != *":$bin_dir:"* ]]; then
        export PATH="$bin_dir:$PATH"
        echo "🛠️ Added $bin_dir to PATH"
    fi
}

# Download and install Roan
install_roan() {
    local version="$1"

    # Check if unzip is installed
    if ! command -v unzip &>/dev/null; then
        echo "Install Failed - 'unzip' is required but not installed. Please install it and try again."
        return 1
    fi

    # Format version
    if [[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        version="v$version"
    elif [[ ! $version =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        version="latest"
    fi

    # Paths and Architecture Setup
    local roan_dir="$HOME/.roan"
    local bin_dir="$roan_dir/bin"
    mkdir -p "$bin_dir"

    local arch=$(get_processor_architecture)
    local os_suffix=$(get_os_artifact_suffix)

    if [[ $arch == "unknown" || $os_suffix == "unknown-os" ]]; then
        echo "Install Failed - Unsupported architecture or operating system."
        return 1
    fi

    local filename="${arch}-${os_suffix}.zip"
    local download_path="$roan_dir/$filename"

    # Remove existing executable
    if [[ -f "$bin_dir/roan" ]]; then
        rm -f "$bin_dir/roan"
    fi

    # Download and install
    echo "📦 Downloading Roan $version for $arch-$os_suffix..."

    local url="https://github.com/roan-rs/roan/releases/$(
        if [[ $version == "latest" ]]; then
            echo "latest/download"
        else
            echo "download/$version"
        fi
    )/$filename"

    # Attempt to download
    if ! curl -L --retry 3 --retry-delay 5 "$url" -o "$download_path"; then
        echo "Install Failed - Could not download from $url"
        return 1
    fi

    # Verify if the downloaded file is a valid ZIP
    if ! file "$download_path" | grep -q 'Zip archive data'; then
        echo "Install Failed - Downloaded file is not in zip format."
        cat "$download_path"  # Display any error message content
        return 1
    fi

    # Extract and setup binary
    if unzip -o "$download_path" -d "$bin_dir"; then
        echo "📚 Unzipped executable to $bin_dir"
        mv "$bin_dir/roan-cli-${arch}-${os_suffix}" "$bin_dir/roan" || {
            echo "Install Failed - Unable to rename the executable"
            return 1
        }
        rm -f "$download_path"
        echo "🎉 Successfully installed Roan $version for $arch-$os_suffix"
        add_bin_dir_to_path "$bin_dir"
    else
        echo "Install Failed - Unable to unzip $download_path"
        return 1
    fi
}

install_roan "$VERSION"

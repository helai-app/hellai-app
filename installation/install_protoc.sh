#!/bin/bash

# Function to install on Ubuntu
install_ubuntu() {
    echo "Updating system..."
    sudo apt update && sudo apt upgrade -y
    echo "Installing protobuf compiler and dev files..."
    sudo apt install -y protobuf-compiler libprotobuf-dev
    echo "Installation complete on Ubuntu."
}

# Function to install on Alpine Linux
install_alpine() {
    echo "Updating system..."
    sudo apk update
    echo "Installing protoc and protobuf-dev..."
    sudo apk add protoc protobuf-dev
    echo "Installation complete on Alpine Linux."
}

# Function to install on macOS
install_macos() {
    echo "Checking for Homebrew..."
    if ! command -v brew &>/dev/null; then
        echo "Homebrew is not installed. Please install Homebrew first."
        exit 1
    fi
    echo "Installing protobuf..."
    brew install protobuf
    echo "Installation complete on macOS."
}

# Function to install on Windows using PowerShell
install_windows() {
    echo "Installing protoc on Windows..."

    # PowerShell command to download and extract protoc
    powershell -Command "
    \$url = 'https://github.com/protocolbuffers/protobuf/releases/download/v21.12/protoc-21.12-win64.zip';
    \$output = 'C:\protoc.zip';
    \$extractPath = 'C:\protoc';

    # Download protoc
    Invoke-WebRequest -Uri \$url -OutFile \$output;

    # Create extract path
    New-Item -ItemType Directory -Path \$extractPath -Force;

    # Extract protoc zip
    Add-Type -AssemblyName 'System.IO.Compression.FileSystem';
    [System.IO.Compression.ZipFile]::ExtractToDirectory(\$output, \$extractPath);

    # Move protoc to system PATH
    \$protocPath = Join-Path \$extractPath 'bin\protoc.exe';
    \$env:Path += ';C:\protoc\bin';
    [Environment]::SetEnvironmentVariable('Path', \$env:Path, [EnvironmentVariableTarget]::Machine);

    # Clean up downloaded zip file
    Remove-Item -Force \$output;

    # Verify installation
    protoc --version
    "

    echo "Installation complete on Windows. Please restart your system for the PATH changes to take effect."
}

# Detect the operating system
OS=$(uname)

case "$OS" in
    "Linux")
        # Check for specific Linux distribution
        if [ -f /etc/alpine-release ]; then
            install_alpine
        elif [ -f /etc/lsb-release ] || [ -f /etc/debian_version ]; then
            install_ubuntu
        else
            echo "Unsupported Linux distribution. Please install dependencies manually."
            exit 1
        fi
        ;;
    "Darwin")
        install_macos
        ;;
    "MINGW"*|"CYGWIN"*)
        install_windows
        ;;
    *)
        echo "Unsupported operating system."
        exit 1
        ;;
esac

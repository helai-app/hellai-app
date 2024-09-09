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

# Function to display Windows instructions
install_windows() {
    echo "Please follow these steps to install protoc on Windows:"
    echo "1. Download the latest version of protoc-xx.y-win64.zip from:"
    echo "   https://github.com/protocolbuffers/protobuf/releases"
    echo "2. Extract the file bin\\protoc.exe."
    echo "3. Add the directory containing protoc.exe to your system PATH."
    echo "4. Verify the installation by running 'protoc --version' in a command prompt."
}


# Prompt the user to select the operating system
echo "Please choose your operating system:"
echo "1) Ubuntu"
echo "2) Alpine Linux"
echo "3) macOS (Silicon)"
echo "4) Windows (Manual installation steps)"

# Read user choice
read -p "Enter the number corresponding to your OS: " os_choice

case "$os_choice" in
    1)
        install_ubuntu
        ;;
    2)
        install_alpine
        ;;
    3)
        install_macos
        ;;
    4)
        install_windows
        ;;
    *)
        echo "Invalid choice. Exiting."
        exit 1
        ;;
esac
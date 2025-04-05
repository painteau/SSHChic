# 🔑 SSHChic

✨ Generate ED25519 keys and find unique patterns in your SSH public keys!

## 🚀 Features

- 🔍 Search public keys using regular expressions
- ⚡ Ultra-fast ED25519 key generation
- 🎯 Streaming mode for continuous searching
- 🔠 Case sensitive/insensitive search support

## 📦 Installation

### 📥 Via GitHub Releases

#### 🪟 Windows
1. Download the latest version from the [releases page](https://github.com/painteau/SSHChic/releases)
2. Extract the ZIP archive to your preferred folder
3. The `sshchic.exe` file is ready to use

#### 🐧 Linux/macOS
1. Download the latest version from the [releases page](https://github.com/painteau/SSHChic/releases)
2. Extract the archive according to its format:
   - For a `.zip` archive:
     ```bash
     unzip sshchic_linux_amd64.zip
     ```
3. Make the file executable:
   ```bash
   chmod +x sshchic
   ```
4. Optional: Move the executable to a directory in your PATH:
   ```bash
   sudo mv sshchic /usr/local/bin/
   ```

### 🛠️ Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/painteau/SSHChic.git
   cd SSHChic
   ```
2. Build the project:
   ```bash
   cargo build --release
   ```

## 🛠️ Usage

Without options, SSHChic generates a key pair and saves them in `id_ed25519` and `id_ed25519.pub` files in the same directory.

### Available Options

```bash
Usage: sshchic [OPTIONS]

Options:
    -i, --insensitive    Case insensitive search
    -r, --regex <PATTERN>    Regex pattern to search for
    -s, --streaming      Continue searching after a match
    -h, --help          Print help
    -V, --version       Print version
```

## 💡 Examples

### 1️⃣ Search for Specific Patterns

```bash
# Search for "prout" or "caca" at the end of the key (case insensitive) or "NeRD" (case sensitive)
# (case insensitive by default, use --insensitive to enable case sensitive search
sshchic -r '(?i)prout$|caca$|(?-i)nErD$'
```

### 2️⃣ Continuous Search with Streaming

```bash
# Search for "marmelade" continuously (case insensitive)
sshchic -s -i -r marmelade
```

## ⚠️ Important Notes

- 🔒 Rewritten in Rust for improved performance and safety
- 💻 Heavy and long CPU usage in streaming mode or with complex patterns

[![Rust Build](https://github.com/painteau/SSHChic/actions/workflows/build-rust.yml/badge.svg)](https://github.com/painteau/SSHChic/actions/workflows/build-rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

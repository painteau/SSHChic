# 🔑 SSHChic

[![Rust Build](https://github.com/painteau/SSHChic/actions/workflows/build-rust.yml/badge.svg)](https://github.com/painteau/SSHChic/actions/workflows/build-rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**A fast, multi-threaded ED25519 SSH key generator that searches for keys matching custom regex patterns.**

Create "vanity" SSH keys with personalized patterns in the public key or fingerprint. Perfect for making your SSH keys memorable, identifiable, or just aesthetically pleasing!

## ✨ Features

- 🔍 **Regex Pattern Matching** - Full regex support for flexible search patterns
- ⚡ **Multi-threaded Performance** - Utilizes all CPU cores for maximum speed (thousands of keys/second)
- 🎯 **Dual Match Modes** - Match against public key or SHA256 fingerprint
- 🔄 **Streaming Mode** - Continuously generate multiple matching keys
- 🔠 **Case-Insensitive Search** - Optional case-insensitive pattern matching
- 📊 **Real-time Monitoring** - Live statistics on key generation rate
- 🛡️ **Cryptographically Secure** - Uses industry-standard ED25519 algorithm
- 🚀 **Cross-platform** - Works on Linux, macOS, and Windows

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

## 🎯 Quick Start

Generate an SSH key with a pattern at the end:

```bash
sshchic --regex "SSH$"
```

This will search until it finds a key ending with "SSH", then save it to:
- `id_ed25519` - Your private key
- `id_ed25519.pub` - Your public key

## 🛠️ Usage

```bash
sshchic [OPTIONS]

Options:
  -r, --regex <PATTERN>    Regex pattern to search for (required)
  -i, --insensitive        Enable case-insensitive matching
  -s, --streaming          Keep searching after finding matches
  -f, --fingerprint        Match against fingerprint instead of public key
  -h, --help              Print help information
  -V, --version           Print version information
```

## 💡 Examples

### Basic Pattern Matching

```bash
# Find a key ending with "SSH"
sshchic --regex "SSH$"

# Find a key containing "github" (case-insensitive)
sshchic --regex "github" --insensitive

# Find a key starting with specific characters
sshchic --regex "^ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAtest"
```

### Fingerprint Matching

```bash
# Search for pattern in the SHA256 fingerprint
sshchic --regex "deadbeef" --fingerprint

# Find a memorable fingerprint
sshchic --regex "1234" --fingerprint
```

### Advanced Patterns

```bash
# Find keys with repeated characters
sshchic --regex "(.)\1\1"  # Three same characters in a row

# Find keys with digits
sshchic --regex "[0-9]{4}"  # Four consecutive digits

# Multiple pattern options
sshchic --regex "(cat|dog|fox)"

# Palindrome pattern
sshchic --regex "(.)(.)\2\1"
```

### Streaming Mode

```bash
# Generate multiple matching keys (displays but doesn't save)
sshchic --regex "test" --streaming

# Press Ctrl+C when you've found enough options
```

### Real-World Use Cases

```bash
# Personal branding - GitHub profile
sshchic --regex "alice" --insensitive

# Server identification
sshchic --regex "prod" --fingerprint

# Team keys
sshchic --regex "devops" --insensitive

# Lucky numbers
sshchic --regex "777"
```

## 📊 Performance

**Typical Performance** (8-core CPU):
- ~10,000 - 50,000 keys/second
- Linear scaling with CPU cores

**Expected Search Times** (approximate):

| Pattern Length | Example | Estimated Time |
|----------------|---------|----------------|
| 1-2 characters | `AB` | < 1 second |
| 3 characters | `cat` | ~5 seconds |
| 4 characters | `test` | ~5 minutes |
| 5 characters | `hello` | ~6 hours |
| 6+ characters | `github` | days to weeks |

**Tip**: Shorter patterns = faster results! See [docs/EXAMPLES.md](docs/EXAMPLES.md) for detailed guidance.

## ⚠️ Important Notes

- **File Overwriting**: Existing `id_ed25519` files will be overwritten without warning
- **Streaming Mode**: Keys are displayed but NOT saved to files in streaming mode
- **CPU Usage**: Will utilize all CPU cores - expect high CPU usage during search
- **Pattern Feasibility**: Very long patterns (6+ characters) may take days or longer
- **Security**: Generated keys are cryptographically secure, but vanity patterns slightly reduce entropy

## 📚 Documentation

- **[Examples Guide](docs/EXAMPLES.md)** - Detailed usage examples and patterns
- **[Architecture](docs/ARCHITECTURE.md)** - Technical design and implementation details
- **[Contributing](CONTRIBUTING.md)** - How to contribute to the project
- **[API Documentation](https://docs.rs/sshchic)** - Rustdoc generated documentation

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development setup
- Code standards
- Testing guidelines
- Pull request process

## 🔒 Security

SSHChic uses:
- **ED25519** - Modern elliptic curve cryptography
- **ChaCha20 CSPRNG** - Cryptographically secure random number generation
- **Audited Libraries** - `ed25519-dalek` and other well-tested crates

**Recommendation**: After generating keys, set proper permissions:
```bash
chmod 600 id_ed25519      # Private key - owner read/write only
chmod 644 id_ed25519.pub  # Public key - world readable
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

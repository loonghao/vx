## 🚀 vx {{VERSION}}

Universal Development Tool Manager - Manage multiple development tools and their versions.

### 📦 Download Options

Choose the appropriate archive for your platform:

- **Linux (x86_64)**: `vx-linux-amd64.tar.gz`
- **Windows (x86_64)**: `vx-windows-amd64.zip`  
- **macOS Intel (x86_64)**: `vx-macos-amd64.tar.gz`
- **macOS Apple Silicon (ARM64)**: `vx-macos-arm64.tar.gz`

### 🔐 Verification

All files include SHA256 checksums in `checksums.txt` for verification:

```bash
# Verify downloaded file
sha256sum -c checksums.txt
```

### 📋 Installation

#### Linux / macOS
```bash
# Download and extract
curl -L -o vx-linux-amd64.tar.gz https://github.com/loonghao/vx/releases/download/{{VERSION}}/vx-linux-amd64.tar.gz
tar -xzf vx-linux-amd64.tar.gz

# Install to system
sudo mv vx /usr/local/bin/

# Verify installation
vx --version
```

#### Windows
```powershell
# Download and extract the zip file
# Move vx.exe to a directory in your PATH
# Verify installation
vx --version
```

### 🆕 What's New

{{CHANGELOG}}

### 🐛 Bug Reports

If you encounter any issues, please report them at: https://github.com/loonghao/vx/issues

### 💝 Support

If you find vx useful, please consider:
- ⭐ Starring the repository
- 🐛 Reporting bugs
- 💡 Suggesting features
- 🤝 Contributing code

---

**Made with ❤️ for developers, by developers**

# Troubleshooting

Common issues and solutions.

## Installation Issues

### "command not found: vx"

The vx binary is not in your PATH.

**Solution:**

1. Check installation location
2. Add to PATH:

   ```bash
   export PATH="$HOME/.local/bin:$PATH"
   ```

3. Restart your terminal

### Permission denied during installation

**Solution:**

```bash
# Don't use sudo with the install script
# Instead, ensure ~/.local/bin exists and is writable
mkdir -p ~/.local/bin
chmod 755 ~/.local/bin
```

## Tool Installation Issues

### "Failed to download"

Network or URL issues.

**Solutions:**

1. Check internet connection
2. Try again (transient error)
3. Check proxy settings:

   ```bash
   export HTTP_PROXY=http://proxy:port
   export HTTPS_PROXY=http://proxy:port
   ```

### "Version not found"

The specified version doesn't exist.

**Solutions:**

1. Check available versions:

   ```bash
   vx versions <tool>
   ```

2. Use a valid version specifier
3. Clear version cache:

   ```bash
   vx clean --cache
   ```

### "Checksum mismatch"

Downloaded file is corrupted.

**Solutions:**

1. Clear cache and retry:

   ```bash
   vx clean --cache
   vx install <tool>
   ```

2. Check disk space

## Execution Issues

### "Tool not found" after installation

The tool was installed but can't be found.

**Solutions:**

1. Check installation:

   ```bash
   vx which <tool>
   ```

2. Verify store directory:

   ```bash
   ls ~/.local/share/vx/store/<tool>/
   ```

3. Try reinstalling:

   ```bash
   vx install <tool> --force
   ```

### Wrong version being used

**Solutions:**

1. Check version resolution:

   ```bash
   vx --verbose <tool> --version
   ```

2. Check `vx.toml` in current directory
3. Check global config:

   ```bash
   vx config show
   ```

### "Permission denied" when running tool

**Solutions:**

1. Check file permissions:

   ```bash
   ls -la ~/.local/share/vx/store/<tool>/<version>/
   ```

2. Fix permissions:

   ```bash
   chmod +x ~/.local/share/vx/store/<tool>/<version>/<binary>
   ```

## Configuration Issues

### "vx.toml not found"

**Solutions:**

1. Check current directory
2. Create configuration:

   ```bash
   vx init
   ```

### "Invalid configuration"

TOML syntax error.

**Solutions:**

1. Validate TOML syntax
2. Check for common issues:
   - Missing quotes around strings
   - Incorrect indentation
   - Invalid characters

### Environment variables not set

**Solutions:**

1. Check `[env]` section in `vx.toml`
2. Verify with:

   ```bash
   vx dev -c "env | grep MY_VAR"
   ```

## Shell Integration Issues

### Completions not working

**Solutions:**

1. Regenerate completions:

   ```bash
   vx shell completions bash > ~/.local/share/bash-completion/completions/vx
   ```

2. Restart shell
3. For Zsh, ensure compinit is called

### Auto-switching not working

**Solutions:**

1. Verify shell integration is set up:

   ```bash
   echo $VX_ENV
   ```

2. Re-add to shell profile:

   ```bash
   eval "$(vx shell init bash)"
   ```

## Performance Issues

### Slow startup

**Solutions:**

1. Use lazy loading in shell profile
2. Cache init script:

   ```bash
   vx shell init bash > ~/.vx-init.sh
   source ~/.vx-init.sh
   ```

### High disk usage

**Solutions:**

1. Check usage:

   ```bash
   vx stats
   ```

2. Clean up:

   ```bash
   vx clean --all
   ```

3. Remove unused versions:

   ```bash
   vx uninstall <tool> <version>
   ```

## Getting Help

### Debug Output

Enable debug output for detailed information:

```bash
vx --debug <command>
```

### Verbose Output

```bash
vx --verbose <command>
```

### Check Version

```bash
vx --version
```

### Report Issues

If you can't resolve an issue:

1. Search [existing issues](https://github.com/loonghao/vx/issues)
2. Create a new issue with:
   - vx version
   - OS and shell
   - Steps to reproduce
   - Error messages
   - Debug output

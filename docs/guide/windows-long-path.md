# Windows Long Path Support

Windows has a traditional path length limit of 260 characters (`MAX_PATH`). This can cause issues when working with deeply nested directory structures, especially common in Node.js projects with `node_modules` dependencies.

vx provides built-in support to handle long paths on Windows, ensuring smooth operation even with complex project structures.

## The Problem

When you install npm packages with deep dependency trees, the resulting path can easily exceed 260 characters:

```
C:\Users\username\.vx\store\node\20.0.0\node_modules\@scope\package\node_modules\another\node_modules\deeply\nested\file.js
```

This can cause:
- Installation failures
- File access errors
- Extraction failures from archives

## Solutions

vx addresses this problem through multiple layers:

### 1. Automatic Detection and Warning

During installation, vx automatically checks if Windows long path support is enabled. If not, it displays helpful instructions:

```powershell
# When running install.ps1
⚠️  Windows Long Path Support is NOT enabled

vx may encounter issues with deep directory paths (>260 characters),
especially when installing npm packages with nested dependencies.

To enable long path support (recommended):

Option 1: Run this PowerShell command (requires Administrator):
  New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
      -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force

Option 2: Via Group Policy (Windows 10 Pro/Enterprise):
  1. Open gpedit.msc
  2. Navigate to: Computer Configuration > Administrative Templates > System > Filesystem
  3. Enable 'Enable Win32 long paths'

Option 3: Use a shorter VX_HOME path:
  $env:VX_HOME = "C:\vx"
```

### 2. Built-in Extended Path Support

Even without system-level long path support, vx uses the Windows extended-length path prefix (`\\?\`) internally when extracting archives. This allows paths up to 32,767 characters.

When a path approaches or exceeds the 260-character limit:
- vx logs a warning
- Automatically converts the path to extended format (`\\?\C:\...`)
- Continues operation successfully

### 3. Short Base Path Option

You can configure vx to use a shorter base path to minimize path length issues:

```powershell
# Set a shorter VX_HOME directory
$env:VX_HOME = "C:\vx"

# Add to your PowerShell profile for persistence
Add-Content $PROFILE '$env:VX_HOME = "C:\vx"'
```

## Enabling Windows Long Path Support

### Method 1: PowerShell (Recommended)

Run PowerShell as Administrator and execute:

```powershell
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
    -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

### Method 2: Group Policy (Windows 10/11 Pro/Enterprise)

1. Press `Win + R`, type `gpedit.msc`, and press Enter
2. Navigate to: **Computer Configuration** → **Administrative Templates** → **System** → **Filesystem**
3. Double-click **Enable Win32 long paths**
4. Select **Enabled** and click **OK**

### Method 3: Registry Editor

1. Press `Win + R`, type `regedit`, and press Enter
2. Navigate to: `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\FileSystem`
3. Find or create `LongPathsEnabled` (DWORD)
4. Set value to `1`

> **Note:** After enabling long path support, restart your terminal or reboot Windows for changes to take effect.

## Checking Current Status

You can check if long path support is enabled:

```powershell
# Check registry value
Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name "LongPathsEnabled"

# Expected output when enabled:
# LongPathsEnabled : 1
```

## API Reference (for Developers)

vx provides a `vx_paths::windows` module for handling long paths programmatically:

```rust
use vx_paths::windows::{
    to_long_path,           // Convert to \\?\ format
    from_long_path,         // Remove \\?\ prefix
    check_path_length,      // Check if path exceeds limits
    is_long_path_enabled,   // Check system setting
    PathLengthStatus,       // Safe/Warning/TooLong
};

// Convert path for extended length support
let long_path = to_long_path(&my_path);

// Check path length status
match check_path_length(&path) {
    PathLengthStatus::Safe => { /* OK */ }
    PathLengthStatus::Warning { length, .. } => { 
        println!("Path approaching limit: {} chars", length);
    }
    PathLengthStatus::TooLong { length, .. } => {
        println!("Path exceeds limit: {} chars", length);
    }
}

// Check if system has long path support enabled
if !is_long_path_enabled() {
    println!("Consider enabling Windows long path support");
}
```

## Best Practices

1. **Enable system-level support**: This is the most comprehensive solution
2. **Use short base paths**: Set `VX_HOME` to a short path like `C:\vx`
3. **Avoid deeply nested structures**: When possible, flatten your project structure
4. **Use pnpm**: pnpm's flat `node_modules` structure significantly reduces path lengths

## Troubleshooting

### Error: "The filename or extension is too long"

This error occurs when Windows cannot handle a long path. Solutions:
1. Enable long path support (see above)
2. Set `VX_HOME` to a shorter path
3. vx will attempt to use `\\?\` prefix automatically

### Warning: "Path length approaching Windows limit"

vx detected a path that's getting close to 260 characters. While it will still work, consider:
1. Enabling system-level long path support
2. Using a shorter `VX_HOME` path

### Archive extraction fails silently

Some archive operations may fail without clear error messages. vx now:
1. Logs warnings for paths approaching the limit
2. Automatically uses extended-length paths when needed
3. Reports success/failure with path information

## Related Documentation

- [Installation Guide](./installation.md)
- [Configuration](./configuration.md)
- [Environment Variables](../config/env-vars.md)

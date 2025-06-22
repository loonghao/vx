# VX Git Hooks Configuration
# You can override these settings by setting environment variables

# Enable quick tests on affected modules (default: false)
# $env:VX_QUICK_TEST = "true"

# Enable strict mode - fail on any warnings (default: false)
# $env:VX_STRICT_MODE = "true"

# Enable automatic formatting (default: true)
# $env:VX_AUTO_FIX = "true"

# To use these settings in PowerShell:
# $env:VX_QUICK_TEST = "true"
# git commit -m "your message"

# Or create a PowerShell profile function:
# function Git-Commit-Strict { $env:VX_STRICT_MODE = "true"; git commit @args }
# Set-Alias gcs Git-Commit-Strict

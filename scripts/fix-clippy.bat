@echo off
REM Clippy fix script for Windows
REM Automatically fixes common clippy warnings and runs checks

setlocal enabledelayedexpansion

REM Colors (using echo with special characters)
set "GREEN=[32m"
set "YELLOW=[33m"
set "BLUE=[34m"
set "RED=[31m"
set "NC=[0m"

:show_help
if "%1"=="--help" goto :help
if "%1"=="-h" goto :help
goto :main

:help
echo Clippy Fix Script for vx (Windows)
echo.
echo Usage: %0 [OPTIONS]
echo.
echo OPTIONS:
echo     -h, --help          Show this help message
echo     -f, --fix           Automatically fix clippy warnings
echo     -c, --check         Only check for clippy warnings (no fixes)
echo     -w, --workspace     Run on entire workspace
echo     --all-features      Check with all features enabled
echo.
echo EXAMPLES:
echo     %0                          # Basic clippy check
echo     %0 --fix                    # Fix clippy warnings automatically
echo     %0 --workspace --fix        # Fix entire workspace
echo.
goto :eof

:log_info
echo %BLUE%[INFO]%NC% %~1
goto :eof

:log_success
echo %GREEN%[SUCCESS]%NC% %~1
goto :eof

:log_warning
echo %YELLOW%[WARNING]%NC% %~1
goto :eof

:log_error
echo %RED%[ERROR]%NC% %~1
goto :eof

:check_cargo_config
call :log_info "Checking Cargo configuration..."

if exist ".cargo\config.toml" (
    findstr /C:"jobs = 0" ".cargo\config.toml" >nul
    if !errorlevel! equ 0 (
        call :log_warning "Found 'jobs = 0' in .cargo\config.toml, this may cause issues"
        call :log_info "Consider removing this line or setting it to a specific number"
    )
    
    findstr /C:"rustc-wrapper.*sccache" ".cargo\config.toml" >nul
    if !errorlevel! equ 0 (
        where sccache >nul 2>&1
        if !errorlevel! neq 0 (
            call :log_warning "sccache is configured but not installed"
            call :log_info "Install with: cargo install sccache"
        )
    )
)
goto :eof

:run_clippy_check
call :log_info "Running clippy check..."

cargo clippy %CLIPPY_ARGS% -- -D warnings
if %errorlevel% equ 0 (
    call :log_success "Clippy check passed!"
    exit /b 0
) else (
    call :log_error "Clippy check failed!"
    exit /b 1
)

:run_clippy_fix
call :log_info "Running clippy fix..."

cargo clippy %CLIPPY_ARGS% --fix --allow-dirty --allow-staged -- -D warnings
if %errorlevel% equ 0 (
    call :log_success "Clippy fix completed!"
    exit /b 0
) else (
    call :log_error "Clippy fix failed!"
    exit /b 1
)

:main
REM Check if we're in a Rust project
if not exist "Cargo.toml" (
    call :log_error "Cargo.toml not found. Please run this script from the project root."
    exit /b 1
)

REM Parse command line arguments
set "FIX_MODE=false"
set "CHECK_ONLY=false"
set "WORKSPACE=false"
set "ALL_FEATURES=false"
set "CLIPPY_ARGS="

:parse_args
if "%1"=="" goto :args_done
if "%1"=="-f" set "FIX_MODE=true" & shift & goto :parse_args
if "%1"=="--fix" set "FIX_MODE=true" & shift & goto :parse_args
if "%1"=="-c" set "CHECK_ONLY=true" & shift & goto :parse_args
if "%1"=="--check" set "CHECK_ONLY=true" & shift & goto :parse_args
if "%1"=="-w" set "WORKSPACE=true" & shift & goto :parse_args
if "%1"=="--workspace" set "WORKSPACE=true" & shift & goto :parse_args
if "%1"=="--all-features" set "ALL_FEATURES=true" & shift & goto :parse_args
shift
goto :parse_args

:args_done

call :log_info "Starting clippy analysis for vx project..."

REM Check Cargo configuration
call :check_cargo_config

REM Build clippy arguments
if "%WORKSPACE%"=="true" (
    set "CLIPPY_ARGS=!CLIPPY_ARGS! --workspace"
)

if "%ALL_FEATURES%"=="true" (
    set "CLIPPY_ARGS=!CLIPPY_ARGS! --all-features"
)

set "CLIPPY_ARGS=!CLIPPY_ARGS! --all-targets"

REM Run clippy
if "%FIX_MODE%"=="true" (
    if "%CHECK_ONLY%"=="false" (
        call :log_info "Running clippy in fix mode..."
        
        REM First try to fix automatically
        call :run_clippy_fix
        if !errorlevel! equ 0 (
            call :log_success "Automatic fixes applied successfully"
        ) else (
            call :log_warning "Some issues couldn't be fixed automatically"
        )
        
        REM Then run check to see remaining issues
        call :log_info "Checking for remaining issues..."
        call :run_clippy_check
    )
) else (
    call :log_info "Running clippy in check mode..."
    call :run_clippy_check
)

REM Additional checks
call :log_info "Running additional code quality checks..."

REM Check formatting
cargo fmt -- --check >nul 2>&1
if %errorlevel% equ 0 (
    call :log_success "Code formatting is correct"
) else (
    call :log_warning "Code formatting issues found. Run 'cargo fmt' to fix."
)

call :log_success "Clippy analysis completed!"

endlocal

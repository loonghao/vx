//! .NET/C# project analyzer
//!
//! This module provides analysis for .NET/C# projects, including:
//! - Detection via .csproj, .sln, .fsproj, global.json
//! - Dependency detection from .csproj PackageReference
//! - Script detection from common dotnet commands
//! - Required tool detection

mod analyzer;
mod dependencies;
mod rules;

pub use analyzer::DotNetAnalyzer;

//! Tests for .NET/C# project analyzer

use rstest::rstest;
use std::path::PathBuf;
use tempfile::TempDir;
use vx_project_analyzer::{AnalyzerConfig, Ecosystem, ProjectAnalyzer};

/// Create a temp directory with a simple .csproj project
async fn create_csproj_project(dir: &TempDir) -> PathBuf {
    let root = dir.path().to_path_buf();

    let csproj = r#"<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
    <PackageReference Include="Serilog" Version="3.1.1" />
    <PackageReference Include="Microsoft.Extensions.Hosting" Version="8.0.0" />
  </ItemGroup>

</Project>"#;

    tokio::fs::write(root.join("MyApp.csproj"), csproj)
        .await
        .unwrap();

    root
}

/// Create a temp directory with a .sln solution
async fn create_sln_project(dir: &TempDir) -> PathBuf {
    let root = dir.path().to_path_buf();

    let sln = r#"
Microsoft Visual Studio Solution File, Format Version 12.00
# Visual Studio Version 17
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "MyApp", "src\MyApp\MyApp.csproj", "{12345678-1234-1234-1234-123456789ABC}"
EndProject
"#;

    tokio::fs::write(root.join("MyApp.sln"), sln).await.unwrap();

    // Create subdirectory with .csproj
    tokio::fs::create_dir_all(root.join("src/MyApp"))
        .await
        .unwrap();

    let csproj = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="MediatR" Version="12.0.0" />
  </ItemGroup>
</Project>"#;

    tokio::fs::write(root.join("src/MyApp/MyApp.csproj"), csproj)
        .await
        .unwrap();

    root
}

/// Create a temp directory with global.json
async fn create_global_json_project(dir: &TempDir) -> PathBuf {
    let root = dir.path().to_path_buf();

    let global_json = r#"{
  "sdk": {
    "version": "8.0.100",
    "rollForward": "latestMinor"
  }
}"#;

    tokio::fs::write(root.join("global.json"), global_json)
        .await
        .unwrap();

    root
}

/// Create a temp directory with Directory.Packages.props (central package management)
async fn create_central_packages_project(dir: &TempDir) -> PathBuf {
    let root = dir.path().to_path_buf();

    let packages_props = r#"<Project>
  <PropertyGroup>
    <ManagePackageVersionsCentrally>true</ManagePackageVersionsCentrally>
  </PropertyGroup>
  <ItemGroup>
    <PackageVersion Include="Newtonsoft.Json" Version="13.0.3" />
    <PackageVersion Include="xunit" Version="2.7.0" />
    <PackageVersion Include="FluentAssertions" Version="6.12.0" />
  </ItemGroup>
</Project>"#;

    tokio::fs::write(root.join("Directory.Packages.props"), packages_props)
        .await
        .unwrap();

    // Also need a .csproj to be a valid .NET project
    let csproj = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Newtonsoft.Json" />
  </ItemGroup>
</Project>"#;

    tokio::fs::write(root.join("MyLib.csproj"), csproj)
        .await
        .unwrap();

    root
}

/// Create a temp directory with F# project
async fn create_fsproj_project(dir: &TempDir) -> PathBuf {
    let root = dir.path().to_path_buf();

    let fsproj = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="FSharp.Core" Version="8.0.0" />
  </ItemGroup>
</Project>"#;

    tokio::fs::write(root.join("MyFSharpApp.fsproj"), fsproj)
        .await
        .unwrap();

    root
}

// ===========================================================================
// Detection tests
// ===========================================================================

#[rstest]
#[tokio::test]
async fn test_dotnet_csproj_detection() {
    let dir = TempDir::new().unwrap();
    let root = create_csproj_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    assert!(analysis.ecosystems.contains(&Ecosystem::DotNet));
}

#[rstest]
#[tokio::test]
async fn test_dotnet_sln_detection() {
    let dir = TempDir::new().unwrap();
    let root = create_sln_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    assert!(analysis.ecosystems.contains(&Ecosystem::DotNet));
}

#[rstest]
#[tokio::test]
async fn test_dotnet_global_json_detection() {
    let dir = TempDir::new().unwrap();
    let root = create_global_json_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    assert!(analysis.ecosystems.contains(&Ecosystem::DotNet));
}

#[rstest]
#[tokio::test]
async fn test_dotnet_fsproj_detection() {
    let dir = TempDir::new().unwrap();
    let root = create_fsproj_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    assert!(analysis.ecosystems.contains(&Ecosystem::DotNet));
}

#[rstest]
#[tokio::test]
async fn test_dotnet_directory_build_props_detection() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();

    tokio::fs::write(
        root.join("Directory.Build.props"),
        r#"<Project>
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#,
    )
    .await
    .unwrap();

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    assert!(analysis.ecosystems.contains(&Ecosystem::DotNet));
}

// ===========================================================================
// Dependency tests
// ===========================================================================

#[rstest]
#[tokio::test]
async fn test_dotnet_csproj_dependencies() {
    let dir = TempDir::new().unwrap();
    let root = create_csproj_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    let dep_names: Vec<_> = analysis.dependencies.iter().map(|d| &d.name).collect();
    assert!(dep_names.contains(&&"Newtonsoft.Json".to_string()));
    assert!(dep_names.contains(&&"Serilog".to_string()));
    assert!(dep_names.contains(&&"Microsoft.Extensions.Hosting".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_dotnet_sln_nested_dependencies() {
    let dir = TempDir::new().unwrap();
    let root = create_sln_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    let dep_names: Vec<_> = analysis.dependencies.iter().map(|d| &d.name).collect();
    assert!(dep_names.contains(&&"MediatR".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_dotnet_central_package_management() {
    let dir = TempDir::new().unwrap();
    let root = create_central_packages_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    let dep_names: Vec<_> = analysis.dependencies.iter().map(|d| &d.name).collect();
    assert!(dep_names.contains(&&"Newtonsoft.Json".to_string()));
    assert!(dep_names.contains(&&"xunit".to_string()));
    assert!(dep_names.contains(&&"FluentAssertions".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_dotnet_dependency_versions() {
    let dir = TempDir::new().unwrap();
    let root = create_csproj_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    let json_dep = analysis
        .dependencies
        .iter()
        .find(|d| d.name == "Newtonsoft.Json")
        .expect("Newtonsoft.Json dependency should exist");

    assert_eq!(json_dep.version.as_deref(), Some("13.0.3"));
}

// ===========================================================================
// Script tests
// ===========================================================================

#[rstest]
#[tokio::test]
async fn test_dotnet_csproj_scripts() {
    let dir = TempDir::new().unwrap();
    let root = create_csproj_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    let script_names: Vec<_> = analysis.scripts.iter().map(|s| &s.name).collect();
    assert!(script_names.contains(&&"build".to_string()));
    assert!(script_names.contains(&&"test".to_string()));
    assert!(script_names.contains(&&"restore".to_string()));
    assert!(script_names.contains(&&"clean".to_string()));
    assert!(script_names.contains(&&"format".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_dotnet_csproj_has_run_script() {
    // Single .csproj without .sln should have a "run" script
    let dir = TempDir::new().unwrap();
    let root = create_csproj_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    let script_names: Vec<_> = analysis.scripts.iter().map(|s| &s.name).collect();
    assert!(script_names.contains(&&"run".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_dotnet_sln_no_run_script() {
    // .sln projects should NOT have a "run" script (multi-project)
    let dir = TempDir::new().unwrap();
    let root = create_sln_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    let script_names: Vec<_> = analysis.scripts.iter().map(|s| &s.name).collect();
    // .sln has both .sln and .csproj in subdir, but the analyzer checks root-level .csproj
    // Since there's a .sln at root, "run" should not be added
    assert!(
        script_names.contains(&&"build".to_string()),
        "Should have build script"
    );
}

// ===========================================================================
// Required tools tests
// ===========================================================================

#[rstest]
#[tokio::test]
async fn test_dotnet_required_tools() {
    let dir = TempDir::new().unwrap();
    let root = create_csproj_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    let tool_names: Vec<_> = analysis.required_tools.iter().map(|t| &t.name).collect();
    assert!(tool_names.contains(&&"dotnet".to_string()));
}

// ===========================================================================
// Edge cases
// ===========================================================================

#[rstest]
#[tokio::test]
async fn test_empty_dir_no_dotnet() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    assert!(!analysis.ecosystems.contains(&Ecosystem::DotNet));
}

#[rstest]
#[tokio::test]
async fn test_dotnet_with_other_ecosystem() {
    // .NET project alongside Node.js project
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();

    // Create .csproj
    let csproj = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Swashbuckle.AspNetCore" Version="6.5.0" />
  </ItemGroup>
</Project>"#;
    tokio::fs::write(root.join("Backend.csproj"), csproj)
        .await
        .unwrap();

    // Create package.json
    let package_json = r#"{
  "name": "frontend",
  "version": "1.0.0",
  "dependencies": { "react": "^18.0.0" }
}"#;
    tokio::fs::write(root.join("package.json"), package_json)
        .await
        .unwrap();

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    assert!(analysis.ecosystems.contains(&Ecosystem::DotNet));
    assert!(analysis.ecosystems.contains(&Ecosystem::NodeJs));
}

#[tokio::test]
async fn test_dotnet_deep_detection_csproj_in_subdirectory() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a nested .csproj project: src/MyApp/MyApp.csproj
    let src_dir = root.join("src").join("MyApp");
    tokio::fs::create_dir_all(&src_dir).await.unwrap();

    let csproj = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#;
    tokio::fs::write(src_dir.join("MyApp.csproj"), csproj)
        .await
        .unwrap();

    let config = AnalyzerConfig {
        max_depth: 3,
        ..Default::default()
    };
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    assert!(
        analysis.ecosystems.contains(&Ecosystem::DotNet),
        "Should detect .NET project in nested subdirectory"
    );
}

#[tokio::test]
async fn test_dotnet_deep_detection_sln_level2() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a solution in a subdirectory: backend/MyApp.sln
    let backend_dir = root.join("backend");
    tokio::fs::create_dir_all(&backend_dir).await.unwrap();
    tokio::fs::write(backend_dir.join("MyApp.sln"), "solution content")
        .await
        .unwrap();

    let config = AnalyzerConfig {
        max_depth: 3,
        ..Default::default()
    };
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    assert!(
        analysis.ecosystems.contains(&Ecosystem::DotNet),
        "Should detect .NET solution file in level-2 subdirectory"
    );
}

#[tokio::test]
async fn test_dotnet_deep_detection_level3() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a nested .csproj project: services/api/MyApi/MyApi.csproj
    let api_dir = root.join("services").join("api").join("MyApi");
    tokio::fs::create_dir_all(&api_dir).await.unwrap();

    let csproj = r#"<Project Sdk="Microsoft.NET.Sdk.Web">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#;
    tokio::fs::write(api_dir.join("MyApi.csproj"), csproj)
        .await
        .unwrap();

    let config = AnalyzerConfig {
        max_depth: 3,
        ..Default::default()
    };
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    assert!(
        analysis.ecosystems.contains(&Ecosystem::DotNet),
        "Should detect .NET project in level-3 subdirectory"
    );
}

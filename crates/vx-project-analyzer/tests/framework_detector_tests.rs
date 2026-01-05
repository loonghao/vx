use rstest::rstest;
use std::fs;
use tempfile::TempDir;
use vx_project_analyzer::frameworks::{FlutterDetector, ReactNativeDetector};
use vx_project_analyzer::{
    AnalyzerConfig, FrameworkDetector, ProjectAnalyzer, Script, ScriptSource,
};

fn write_file(dir: &std::path::Path, name: &str, content: &str) {
    fs::write(dir.join(name), content).expect("write fixture");
}

#[rstest]
#[tokio::test]
async fn detect_react_native_via_package_json() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    write_file(
        root,
        "package.json",
        r#"{
  "name": "rn-app",
  "version": "0.1.0",
  "dependencies": {
    "react-native": "^0.74.0"
  },
  "scripts": {
    "android": "react-native run-android"
  }
}
"#,
    );
    fs::create_dir_all(root.join("android")).unwrap();

    let detector = ReactNativeDetector::new();
    assert!(detector.detect(root));

    let scripts = vec![Script::new(
        "android",
        "react-native run-android",
        ScriptSource::PackageJson,
    )];

    let info = detector.get_info(root).await.unwrap();
    assert_eq!(info.framework.to_string(), "React Native");
    assert_eq!(info.build_tool.as_deref(), Some("react-native-cli"));
    assert!(info.version.as_deref().is_some());
    assert!(info.target_platforms.contains(&"android".to_string()));

    let tools = detector.required_tools(&[], &scripts);
    assert!(tools.iter().any(|t| t.name == "node"));
    assert!(tools.iter().any(|t| t.name == "react-native"));
}

#[rstest]
#[tokio::test]
async fn detect_flutter_via_pubspec() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    write_file(
        root,
        "pubspec.yaml",
        r#"name: flutter_app
version: 0.1.0
environment:
  sdk: '>=3.0.0'
dependencies:
  flutter:
    sdk: flutter
"#,
    );

    fs::create_dir_all(root.join("ios")).unwrap();
    fs::create_dir_all(root.join("android")).unwrap();

    let detector = FlutterDetector::new();
    assert!(detector.detect(root));

    let info = detector.get_info(root).await.unwrap();
    assert_eq!(info.framework.to_string(), "Flutter");
    assert_eq!(info.build_tool.as_deref(), Some("flutter"));
    assert_eq!(
        info.config_path.as_ref().map(|p| p.file_name().unwrap()),
        Some(std::ffi::OsStr::new("pubspec.yaml"))
    );
    assert!(info.target_platforms.contains(&"android".to_string()));
    assert!(info.target_platforms.contains(&"ios".to_string()));

    let tools = detector.required_tools(&[], &[]);
    assert!(tools.iter().any(|t| t.name == "flutter"));
}

#[rstest]
#[tokio::test]
async fn analyzer_collects_new_framework_detectors() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_file(
        root,
        "package.json",
        r#"{ "name": "rn-app", "dependencies": { "react-native": "0.74.0", "@capacitor/core": "6.0.0", "nw": "0.85.0" } }"#,
    );

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(root).await.unwrap();

    let frameworks: Vec<_> = analysis
        .frameworks
        .iter()
        .map(|f| f.framework.to_string())
        .collect();

    assert!(frameworks.contains(&"React Native".to_string()));
    assert!(frameworks.contains(&"Capacitor".to_string()));
    assert!(frameworks.contains(&"NW.js".to_string()));
}

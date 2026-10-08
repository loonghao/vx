//! Archive integrity and mirror descriptors must survive the typed bridge.

use rstest::rstest;
use vx_starlark::StarlarkProvider;
use vx_starlark::provider::InstallLayout;

async fn provider_with_layout(fields: &str) -> StarlarkProvider {
    let content = format!(
        r#"
name = "archive-integrity"
description = "Archive descriptor contract"
runtimes = [{{"name": "archive-integrity", "executable": "example"}}]
def install_layout(_ctx, _version):
    return {{"type": "archive", {fields}}}
"#
    );
    StarlarkProvider::from_content("archive-integrity", &content)
        .await
        .unwrap()
}

#[tokio::test]
async fn integrity_fields_survive_typed_layout_and_flattening() {
    let digest = "12".repeat(32);
    let provider = provider_with_layout(&format!(
        r#""sha256": "{digest}", "mirror_urls": ["https://mirror.example/python.tar.zst"]"#
    ))
    .await;
    let layout = provider.install_layout("3.7.9").await.unwrap().unwrap();
    let flat = layout.to_flat_json();
    assert_eq!(flat["sha256"], digest);
    assert_eq!(
        flat["mirror_urls"],
        serde_json::json!(["https://mirror.example/python.tar.zst"])
    );
}

#[rstest]
#[case("sha256", "123")]
#[case("sha256", "[]")]
#[case("sha256", "None")]
#[case("mirror_urls", "\"https://mirror.example/python.tar.zst\"")]
#[case("mirror_urls", "[\"https://mirror.example/python.tar.zst\", 123]")]
#[case("mirror_urls", "None")]
#[case("url", "123")]
#[case("strip_prefix", "[]")]
#[case("executable_paths", "[\"python.exe\", None]")]
#[case("required_paths", "{\"path\": \"python.exe\"}")]
#[tokio::test]
async fn incorrectly_typed_archive_fields_fail(#[case] field: &str, #[case] value: &str) {
    let provider = provider_with_layout(&format!("\"{field}\": {value}")).await;
    let error = provider.install_layout("3.7.9").await.unwrap_err();
    assert!(
        error.to_string().contains(&format!("field '{field}'")),
        "unexpected diagnostic: {error}"
    );
}

#[test]
fn previously_serialized_archive_layouts_default_integrity_fields() {
    let layout: InstallLayout = serde_json::from_value(serde_json::json!({
        "Archive": {
            "url": null,
            "strip_prefix": null,
            "executable_paths": [],
            "required_paths": []
        }
    }))
    .unwrap();
    match layout {
        InstallLayout::Archive {
            sha256,
            mirror_urls,
            ..
        } => {
            assert!(sha256.is_none());
            assert!(mirror_urls.is_empty());
        }
        other => panic!("unexpected layout: {other:?}"),
    }
}

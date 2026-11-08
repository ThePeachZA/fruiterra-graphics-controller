//! Guards the custom NSIS template.
//!
//! Dioxus CLI 0.7.10 derives the installer product name from the binary target
//! (`fruiterra-gc` -> `FruiterraGc`). The custom template is the only place that
//! keeps the approved display name, so regenerating it from the CLI must not
//! silently change what users see.

#[test]
fn nsis_template_keeps_the_approved_display_name() {
    let template = include_str!("../installer/installer.nsi");

    assert!(template.contains(r#"Name "Fruiterra Graphics Controller""#));
    assert!(template.contains(r#""DisplayName" "Fruiterra Graphics Controller""#));
    assert!(template.contains("Fruiterra Graphics Controller.lnk"));
    assert!(!template.contains(r#"Name "{{product_name}}""#));
}

#[test]
fn nsis_template_still_has_the_cli_placeholders() {
    let template = include_str!("../installer/installer.nsi");

    for placeholder in [
        "{{output_path}}",
        "{{main_binary_path}}",
        "{{main_binary_name}}",
        "{{bundle_id}}",
        "{{version}}",
        "{{webview_install_code}}",
        "{{#each staged_files}}",
    ] {
        assert!(
            template.contains(placeholder),
            "custom NSIS template is missing {placeholder}"
        );
    }
}

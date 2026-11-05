use slot::rocknix::{model_in, running_in};

const OS_RELEASE: &str = "OS_NAME=\"ROCKNIX\"
OS_VERSION=\"20261101\"
OS_BUILD=\"official\"
HW_DEVICE=\"RK3566\"
";

#[test]
fn rocknix_is_recognised_from_its_os_release() {
    let dir = tempfile::tempdir().unwrap();
    let release = dir.path().join("os-release");
    std::fs::write(&release, OS_RELEASE).unwrap();
    assert!(running_in(None, &release));
}

#[test]
fn another_os_release_is_not_rocknix() {
    let dir = tempfile::tempdir().unwrap();
    let release = dir.path().join("os-release");
    std::fs::write(&release, "NAME=\"BaseOS\"\nOS_NAME=\"BaseOS\"\n").unwrap();
    assert!(!running_in(None, &release));
    assert!(!running_in(None, &dir.path().join("missing")));
}

#[test]
fn slot_platform_overrides_the_os_release() {
    let dir = tempfile::tempdir().unwrap();
    let release = dir.path().join("os-release");
    std::fs::write(&release, OS_RELEASE).unwrap();
    assert!(!running_in(Some("baseos"), &release));
    assert!(running_in(Some("ROCKNIX"), &dir.path().join("missing")));
}

#[test]
fn the_device_tree_model_loses_its_trailing_nul() {
    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join("model");
    std::fs::write(&model, b"Powkiddy RGB20SX\0").unwrap();
    assert_eq!(model_in(&model), slot::rocknix::RGB20SX);
    assert_eq!(model_in(&dir.path().join("missing")), "");
}

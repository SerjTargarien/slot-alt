use std::path::Path;
use std::sync::OnceLock;

const OS_RELEASE: &str = "/etc/os-release";
const MODEL: &str = "/sys/firmware/devicetree/base/model";

pub const RGB20SX: &str = "Powkiddy RGB20SX";

pub fn running() -> bool {
    static RUNNING: OnceLock<bool> = OnceLock::new();
    *RUNNING.get_or_init(|| {
        running_in(
            std::env::var("SLOT_PLATFORM").ok().as_deref(),
            Path::new(OS_RELEASE),
        )
    })
}

pub fn running_in(platform: Option<&str>, os_release: &Path) -> bool {
    match platform {
        Some(p) => p.eq_ignore_ascii_case("rocknix"),
        None => std::fs::read_to_string(os_release)
            .ok()
            .and_then(|text| crate::baseos::lookup(&text, "OS_NAME"))
            .is_some_and(|name| name == "ROCKNIX"),
    }
}

pub fn model() -> String {
    model_in(Path::new(MODEL))
}

pub fn model_in(path: &Path) -> String {
    std::fs::read(path)
        .map(|raw| {
            String::from_utf8_lossy(&raw)
                .trim_end_matches(['\0', '\n', ' '])
                .to_string()
        })
        .unwrap_or_default()
}

use std::process::{Command, Stdio};

#[macro_export]
macro_rules! analyzer_found {
    () => {
        $crate::utils::analyzer::analyzer_version().is_some()
    };
}

pub fn analyzer_version() -> Option<String> {
    Command::new("rust-analyzer")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .take_if(|res| res.status.success())
        .and_then(|res| String::from_utf8(res.stdout)
            .map(|output| output.replace("\n", ""))
            .ok()
        )
}

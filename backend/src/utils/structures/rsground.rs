use serde::Serialize;

pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8
}

impl Version {
    pub fn from_string(version: impl ToString) -> Option<Self> {
        version
            .to_string()
            .split('.')
            .collect::<Vec<&str>>()
            .get(0..3)
            .and_then(|parts| Some(
                Self {
                    major: parts.first()?.parse().ok()?,
                    minor: parts.get(1)?.parse().ok()?,
                    patch: parts.get(2)?.parse().ok()?
                }
            ))
    }
}

pub struct Crate {
    pub name: String,
    pub version: Version,
    pub features: Vec<String>
}

pub struct CodeInput {
    pub input: String,
    pub lib: Vec<Crate>
}

#[derive(Serialize)]
pub struct CodeResponse {
    errored: bool,
    exit_code: u8,
    compiler_output: String,
    code_output: String
}

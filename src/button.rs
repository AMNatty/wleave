use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WButton {
    pub label: String,
    pub action: String,
    pub text: String,
    pub keybind: String,
    #[serde(default = "default_justify")]
    pub justify: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
    #[serde(default = "default_circular")]
    pub circular: bool,
    #[serde(default)]
    pub icon: Option<String>,
}

fn default_justify() -> String {
    String::from("center")
}

fn default_circular() -> bool {
    false
}

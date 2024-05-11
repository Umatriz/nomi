#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct ClientSettings {
    pub pixels_per_point: Option<f32>,
}
// Maybe use somekind library to detect this?
pub fn default_pixels_per_point_value() -> f32 {
    if cfg!(target_os = "macos") {
        2.0
    } else {
        1.2
    }
}
impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            pixels_per_point: Some(default_pixels_per_point_value()),
        }
    }
}

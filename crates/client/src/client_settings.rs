#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct ClientSettings {
    pub pixels_per_point: Option<f32>
}
// Maybe use somekind library to detect this?
pub fn default_pixels_per_point_value() -> f32 {
    #[cfg(not(target_os = "macos"))]
    return 1.2;
    // We assume that display dpi on macos is higher, although it can be not true
    #[cfg(target_os = "macos")]
    2.0
}
impl Default for ClientSettings {
    fn default() -> Self {
        Self { pixels_per_point: Some(default_pixels_per_point_value()) }
    }
}
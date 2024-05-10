#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct ClientSettings {
    pub pixels_per_point: Option<f32>
}
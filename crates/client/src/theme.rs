use freya::prelude::{BodyTheme, ButtonTheme, FontTheme, Theme, LIGHT_THEME};
#[allow(non_upper_case_globals)]
pub mod colors {
    /*
    * # Realtime colors template
    * ```no_run
     pub const text: &str = "${text.rgb}";
     pub const background: &str = "${bg.rgb}";
     pub const primary: &str = "${primary.rgb}";
     pub const secondary: &str = "${secondary.rgb}";
     pub const accent: &str = "${accent.rgb}";
    * ```
    */
    pub const text: &str = "rgb(3, 12, 6)";
    pub const background: &str = "rgb(251, 254, 252)";
    pub const primary: &str = "rgb(97, 209, 129)";
    pub const secondary: &str = "rgb(215, 244, 223)";
    pub const accent: &str = "rgb(57, 198, 97)";
}
pub const NOMI_THEME_LIGHT: Theme = Theme {
    body: BodyTheme {
        color: colors::text,
        background: colors::background,
    },
    button: ButtonTheme {
        background: colors::primary,
        hover_background: colors::accent,
        font_theme: FontTheme { color: "white" },
    },
    ..LIGHT_THEME
};

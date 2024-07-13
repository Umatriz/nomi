use eframe::egui::{Response, RichText, Ui, WidgetText};

pub trait UiExt {
    fn ui(&self) -> &Ui;
    fn ui_mut(&mut self) -> &mut Ui;

    fn error_label(&mut self, text: impl Into<String>) -> Response {
        let ui = self.ui_mut();
        ui.label(RichText::new(text).color(ui.visuals().error_fg_color))
    }

    fn warn_icon_with_hover_text(&mut self, text: impl Into<WidgetText>) -> Response {
        let ui = self.ui_mut();
        ui.label(RichText::new("⚠").color(ui.visuals().warn_fg_color)).on_hover_text(text)
    }

    fn warn_label(&mut self, text: impl Into<String>) -> Response {
        let ui = self.ui_mut();
        ui.label(RichText::new(text).color(ui.visuals().warn_fg_color))
    }

    fn warn_label_with_icon_before(&mut self, text: impl Into<String>) -> Response {
        let ui = self.ui_mut();
        ui.label(RichText::new(format!("⚠ {}", text.into())).color(ui.visuals().warn_fg_color))
    }
}

impl UiExt for Ui {
    #[inline]
    fn ui(&self) -> &Ui {
        self
    }

    #[inline]
    fn ui_mut(&mut self) -> &mut Ui {
        self
    }
}

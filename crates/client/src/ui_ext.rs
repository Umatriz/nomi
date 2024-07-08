use eframe::egui::{Response, RichText, Ui};

pub trait UiExt {
    fn ui(&self) -> &Ui;
    fn ui_mut(&mut self) -> &mut Ui;

    fn error_label(&mut self, text: impl Into<String>) -> Response {
        let ui = self.ui_mut();
        ui.label(RichText::new(text).color(ui.visuals().error_fg_color))
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

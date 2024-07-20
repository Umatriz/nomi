use eframe::egui::{self, popup_below_widget, Id, PopupCloseBehavior, Response, RichText, Ui, WidgetText};
use egui_notify::{Toast, Toasts};

pub const TOASTS_ID: &str = "global_egui_notify_toasts";

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

    fn markdown_ui(&mut self, id: egui::Id, markdown: &str) {
        use parking_lot::Mutex;
        use std::sync::Arc;

        let ui = self.ui_mut();
        let commonmark_cache = ui.data_mut(|data| {
            data.get_temp_mut_or_default::<Arc<Mutex<egui_commonmark::CommonMarkCache>>>(egui::Id::new("global_egui_commonmark_cache"))
                .clone()
        });

        let mut locked = commonmark_cache.lock();

        egui_commonmark::CommonMarkViewer::new(id).show(ui, &mut locked, markdown);
    }

    fn toggle_button(&mut self, selected: &mut bool, text: impl Into<WidgetText>) -> Response {
        let mut response = self.ui_mut().button(text);
        if response.clicked() {
            *selected = !*selected;
            response.mark_changed();
        }
        response
    }

    fn button_with_confirm_popup<R>(&mut self, id: Id, button_text: impl Into<WidgetText>, add_content: impl FnOnce(&mut Ui) -> R) -> Response {
        let ui = self.ui_mut();

        let popup_id = ui.id().with("button_confirm_popup").with(id);

        let button = ui.button(button_text);

        if button.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }

        popup_below_widget(ui, popup_id, &button, PopupCloseBehavior::CloseOnClickOutside, |ui| {
            ui.set_min_width(150.0);
            add_content(ui)
        });

        button
    }

    fn toasts(&mut self, writer: impl FnOnce(&mut Toasts) -> &mut Toast) {
        use parking_lot::Mutex;
        use std::sync::Arc;

        let ui = self.ui_mut();
        let toasts = ui.data_mut(|data| data.get_temp_mut_or_default::<Arc<Mutex<Toasts>>>(egui::Id::new(TOASTS_ID)).clone());

        let mut locked = toasts.lock();

        writer(&mut locked);
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

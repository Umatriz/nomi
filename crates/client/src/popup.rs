use eframe::egui::{
    AboveOrBelow, Align, Align2, Area, Context, Frame, Id, Key, Layout, Order, Response, Ui,
};

#[derive(Clone, Default)]
pub struct PopupState(bool);

impl PopupState {
    fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|d| d.get_temp::<Self>(id).unwrap_or_default())
    }

    fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }

    pub fn close(&mut self) {
        self.0 = false;
    }
}

pub fn popup<R>(
    ui: &Ui,
    id: Id,
    response: &Response,
    above_or_below: AboveOrBelow,
    add_contents: impl FnOnce(&mut Ui, &mut PopupState) -> R,
) -> Option<R> {
    let mut state = PopupState::load(ui.ctx(), id);

    if response.clicked() {
        state.0 = !state.0
    }

    if !state.0 {
        return None;
    }

    let (pos, pivot) = match above_or_below {
        AboveOrBelow::Above => (response.rect.left_top(), Align2::LEFT_BOTTOM),
        AboveOrBelow::Below => (response.rect.left_bottom(), Align2::LEFT_TOP),
    };

    let area_response = Area::new(id)
        .order(Order::Foreground)
        .constrain(true)
        .fixed_pos(pos)
        .pivot(pivot)
        .show(ui.ctx(), |ui| {
            let frame = Frame::popup(ui.style());
            let frame_margin = frame.total_margin();
            frame
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                        ui.set_width(response.rect.width() - frame_margin.sum().x);
                        add_contents(ui, &mut state)
                    })
                    .inner
                })
                .inner
        });

    if ui.input(|i| i.key_pressed(Key::Escape))
        || (response.clicked_elsewhere() && area_response.response.clicked_elsewhere())
    {
        state.0 = false
    }

    state.store(ui.ctx(), id);

    Some(area_response.inner)
}

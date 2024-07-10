use egui_task_manager::Progress;

pub struct UnitProgress;

impl Progress for UnitProgress {
    fn apply(&self, current: &mut u32) {
        *current += 1;
    }
}

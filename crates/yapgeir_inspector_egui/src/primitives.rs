use std::{any::Any, borrow::Cow};

use egui::{emath::Numeric, DragValue};

pub fn num_row_ui<T: Numeric, const N: usize>(value: &mut [T; N], ui: &mut egui::Ui, _: egui::Id) {
    for i in 0..N {
        ui.add(DragValue::new(&mut value[i]).speed(0.1));
    }
}

pub fn quad_ui(value: &mut dyn Any, ui: &mut egui::Ui, id: egui::Id) {
    let value = value.downcast_mut::<[[f32; 2]; 4]>().unwrap();

    egui::Grid::new(id).show(ui, |ui| {
        num_row_ui(&mut value[1], ui, id);
        ui.separator();
        num_row_ui(&mut value[2], ui, id);
        ui.end_row();
        num_row_ui(&mut value[0], ui, id);
        ui.separator();
        num_row_ui(&mut value[3], ui, id);
        ui.end_row();
    });
}

pub fn num_vector_ui<T: Numeric, const N: usize>(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    id: egui::Id,
) {
    let value = value.downcast_mut::<[T; N]>().unwrap();
    ui.horizontal(|ui| num_row_ui(value, ui, id));
}

pub fn bool_ui(value: &mut dyn Any, ui: &mut egui::Ui, _: egui::Id) {
    let value = value.downcast_mut::<bool>().unwrap();
    ui.checkbox(value, "");
}

pub fn number_ui<T: egui::emath::Numeric>(value: &mut dyn Any, ui: &mut egui::Ui, _: egui::Id) {
    let value = value.downcast_mut::<T>().unwrap();

    let mut widget = DragValue::new(value);
    widget = widget.speed(0.1);
    ui.add(widget);
}

pub fn string_ui(value: &mut dyn Any, ui: &mut egui::Ui, _: egui::Id) {
    let value = value.downcast_mut::<String>().unwrap();

    if value.contains('\n') {
        ui.text_edit_multiline(value);
    } else {
        ui.text_edit_singleline(value);
    }
}

pub fn cow_str_ui(value: &mut dyn Any, ui: &mut egui::Ui, _: egui::Id) {
    let value = value.downcast_mut::<Cow<str>>().unwrap();
    let mut clone = value.to_string();

    let changed = if value.contains('\n') {
        ui.text_edit_multiline(&mut clone).changed()
    } else {
        ui.text_edit_singleline(&mut clone).changed()
    };

    if changed {
        *value = Cow::Owned(clone);
    }
}

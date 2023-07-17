mod primitives;

use std::{
    any::{type_name, Any, TypeId},
    borrow::Cow,
    ffi::OsString,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use egui::{CollapsingHeader, Grid};
use yapgeir_realm::{Realm, ResMut};
use yapgeir_reflection::{
    bevy_reflect::{
        self, std_traits::ReflectDefault, Array, DynamicEnum, DynamicStruct, DynamicTuple,
        DynamicVariant, Enum, List, Map, Reflect, ReflectMut, Struct, Tuple, TupleStruct, TypeInfo,
        TypeRegistry, VariantInfo, VariantType,
    },
    RealmExtensions, Reflection,
};

type GuiElementMutFn = fn(value: &mut dyn Any, ui: &mut egui::Ui, id: egui::Id);

#[derive(Clone)]
pub struct GuiElement {
    pub fn_mut: GuiElementMutFn,
}

fn maybe_grid(i: usize, ui: &mut egui::Ui, id: egui::Id, mut f: impl FnMut(&mut egui::Ui, bool)) {
    match i {
        0 => {}
        1 => f(ui, false),
        _ => Grid::new(id).show(ui, |ui| f(ui, true)).inner,
    };
}

fn maybe_grid_label_if(
    i: usize,
    ui: &mut egui::Ui,
    id: egui::Id,
    always_show_label: bool,
    mut f: impl FnMut(&mut egui::Ui, bool),
) {
    match i {
        0 => {}
        1 if !always_show_label => f(ui, false),
        _ => Grid::new(id).show(ui, |ui| f(ui, true)).inner,
    };
}

fn get_default_value_for(
    type_registry: &TypeRegistry,
    type_id: TypeId,
) -> Option<Box<dyn Reflect>> {
    if let Some(reflect_default) = type_registry.get_type_data::<ReflectDefault>(type_id) {
        return Some(reflect_default.default());
    }

    None
}

fn variant_constructable<'a>(
    type_registry: &TypeRegistry,
    variant: &'a VariantInfo,
) -> Result<(), Vec<&'a str>> {
    let type_id_is_constructable = |type_id: TypeId| {
        type_registry
            .get_type_data::<ReflectDefault>(type_id)
            .is_some()
    };

    let unconstructable_fields: Vec<&'a str> = match variant {
        VariantInfo::Struct(variant) => variant
            .iter()
            .filter_map(|field| {
                (!type_id_is_constructable(field.type_id())).then_some(field.type_name())
            })
            .collect(),
        VariantInfo::Tuple(variant) => variant
            .iter()
            .filter_map(|field| {
                (!type_id_is_constructable(field.type_id())).then_some(field.type_name())
            })
            .collect(),
        VariantInfo::Unit(_) => return Ok(()),
    };

    if unconstructable_fields.is_empty() {
        Ok(())
    } else {
        Err(unconstructable_fields)
    }
}

fn construct_default_variant(
    type_registry: &TypeRegistry,
    variant: &VariantInfo,
    ui: &mut egui::Ui,
) -> Result<DynamicEnum, ()> {
    let dynamic_variant = match variant {
        VariantInfo::Struct(struct_info) => {
            let mut dynamic_struct = DynamicStruct::default();
            for field in struct_info.iter() {
                let field_default_value =
                    match get_default_value_for(type_registry, field.type_id()) {
                        Some(value) => value,
                        None => {
                            return Err(());
                        }
                    };
                dynamic_struct.insert_boxed(field.name(), field_default_value);
            }
            DynamicVariant::Struct(dynamic_struct)
        }
        VariantInfo::Tuple(tuple_info) => {
            let mut dynamic_tuple = DynamicTuple::default();
            for field in tuple_info.iter() {
                let field_default_value =
                    match get_default_value_for(type_registry, field.type_id()) {
                        Some(value) => value,
                        None => {
                            return Err(());
                        }
                    };
                dynamic_tuple.insert_boxed(field_default_value);
            }
            DynamicVariant::Tuple(dynamic_tuple)
        }
        VariantInfo::Unit(_) => DynamicVariant::Unit,
    };
    let dynamic_enum = DynamicEnum::new(variant.name(), dynamic_variant);
    Ok(dynamic_enum)
}

// TODO: register GuiElement for primitive types
pub fn ui_for_reflect_mut(
    type_registry: &TypeRegistry,
    value: ReflectMut,
    ui: &mut egui::Ui,
    id: egui::Id,
) {
    match value {
        ReflectMut::Struct(value) => ui_for_struct(type_registry, value, ui, id),
        ReflectMut::TupleStruct(value) => ui_for_tuple_struct(type_registry, value, ui, id),
        ReflectMut::Tuple(value) => ui_for_tuple(type_registry, value, ui, id),
        ReflectMut::List(value) => ui_for_list(type_registry, value, ui, id),
        ReflectMut::Array(value) => ui_for_array(type_registry, value, ui, id),
        ReflectMut::Map(value) => ui_for_reflect_map(type_registry, value, ui, id),
        ReflectMut::Enum(value) => ui_for_enum(type_registry, value, ui, id),
        ReflectMut::Value(_) => {
            // Values should be processed by s.fn_mut, if we get here,
            // it means we are processing a data type for which a ui representation
            // was not
        }
        _ => {}
    };
}

/// Draw UI for any value that implements Reflect.
pub fn ui_for_reflect(
    type_registry: &TypeRegistry,
    value: &mut dyn Reflect,
    ui: &mut egui::Ui,
    id: egui::Id,
) {
    // There are specific drawing implementations for primitives, check them first
    if let Some(s) = type_registry.get_type_data::<GuiElement>(Any::type_id(value)) {
        (s.fn_mut)(value.as_any_mut(), ui, id);
        return;
    }

    ui_for_reflect_mut(type_registry, value.reflect_mut(), ui, id);
}

fn ui_for_list(type_registry: &TypeRegistry, list: &mut dyn List, ui: &mut egui::Ui, id: egui::Id) {
    ui.vertical(|ui| {
        let len = list.len();
        for i in 0..len {
            let val = list.get_mut(i).unwrap();
            ui.horizontal(|ui| {
                ui_for_reflect(type_registry, val, ui, id.with(i));
            });

            if i != len - 1 {
                ui.separator();
            }
        }

        let Some(TypeInfo::List(info)) = list.get_represented_type_info() else {
            return;
        };

        ui.vertical_centered_justified(|ui| {
            if ui.button("+").clicked() {
                let default =
                    get_default_value_for(type_registry, info.item_type_id()).or_else(|| {
                        let last = len.checked_sub(1)?;
                        Some(Reflect::clone_value(list.get(last)?))
                    });

                if let Some(new_value) = default {
                    list.push(new_value);
                }
            }
        });
    });
}

fn ui_for_array(
    type_registry: &TypeRegistry,
    array: &mut dyn Array,
    ui: &mut egui::Ui,
    id: egui::Id,
) {
    ui.vertical(|ui| {
        let len = array.len();
        for i in 0..len {
            let val = array.get_mut(i).unwrap();
            ui.horizontal(|ui| {
                ui_for_reflect(type_registry, val, ui, id.with(i));
            });

            if i != len - 1 {
                ui.separator();
            }
        }
    });
}

fn ui_for_reflect_map(
    type_registry: &TypeRegistry,
    map: &mut dyn Map,
    ui: &mut egui::Ui,
    id: egui::Id,
) {
    egui::Grid::new(id).show(ui, |ui| {
        for (i, (key, value)) in map.iter().enumerate() {
            // FIXME: get change tracking back
            let mut key = key.clone_value();
            let mut value = key.clone_value();

            ui_for_reflect(type_registry, key.as_mut(), ui, id.with(i));
            ui_for_reflect(type_registry, value.as_mut(), ui, id.with(i));
            ui.end_row();
        }
    });
}

fn ui_for_enum(
    type_registry: &TypeRegistry,
    value: &mut dyn Enum,
    ui: &mut egui::Ui,
    id: egui::Id,
) {
    let Some(type_info) = value.get_represented_type_info() else {
        ui.label("Unrepresentable");
        return;
    };

    let type_info = match type_info {
        TypeInfo::Enum(info) => info,
        _ => unreachable!("invalid reflect impl: type info mismatch"),
    };

    let mut changed = false;

    ui.vertical(|ui| {
        let changed_variant =
            ui_for_enum_variant_select(type_registry, id, ui, value.variant_index(), type_info);
        if let Some((_new_variant, dynamic_enum)) = changed_variant {
            changed = true;
            value.apply(&dynamic_enum);
        }
        let variant_index = value.variant_index();

        let always_show_label = matches!(value.variant_type(), VariantType::Struct);

        maybe_grid_label_if(value.field_len(), ui, id, always_show_label, |ui, label| {
            (0..value.field_len()).for_each(|i| {
                if label {
                    if let Some(name) = value.name_at(i) {
                        ui.label(name);
                    } else {
                        ui.label(i.to_string());
                    }
                }
                let field_value = value
                    .field_at_mut(i)
                    .expect("invalid reflect impl: field len");
                ui_for_reflect(type_registry, field_value, ui, id.with(i));
                ui.end_row();
            })
        });
    });
}

fn ui_for_enum_variant_select(
    type_registry: &TypeRegistry,
    id: egui::Id,
    ui: &mut egui::Ui,
    active_variant_idx: usize,
    info: &bevy_reflect::EnumInfo,
) -> Option<(usize, DynamicEnum)> {
    let mut changed_variant = None;

    ui.horizontal(|ui| {
        egui::ComboBox::new(id.with("select"), "")
            .selected_text(info.variant_names()[active_variant_idx])
            .show_ui(ui, |ui| {
                for (i, variant) in info.iter().enumerate() {
                    let variant_name = variant.name();
                    let is_active_variant = i == active_variant_idx;

                    let variant_is_constructable = variant_constructable(type_registry, variant);

                    ui.add_enabled_ui(variant_is_constructable.is_ok(), |ui| {
                        let variant_label_response: egui::Response =
                            ui.selectable_label(is_active_variant, variant_name);

                        if variant_label_response.clicked() {
                            if let Ok(dynamic_enum) =
                                construct_default_variant(type_registry, variant, ui)
                            {
                                changed_variant = Some((i, dynamic_enum));
                            };
                        }
                    });
                }

                false
            });
    });

    changed_variant
}

fn ui_for_tuple_struct(
    type_registry: &TypeRegistry,
    value: &mut dyn TupleStruct,
    ui: &mut egui::Ui,
    id: egui::Id,
) {
    (0..value.field_len()).for_each(|i| {
        ui.horizontal(|ui| {
            if value.field_len() > 1 {
                ui.label(format!("{i}:"));
            }
            let field = value.field_mut(i).unwrap();
            ui_for_reflect(type_registry, field, ui, id.with(i));
        });
    })
}

fn ui_for_tuple(
    type_registry: &TypeRegistry,
    value: &mut dyn Tuple,
    ui: &mut egui::Ui,
    id: egui::Id,
) {
    maybe_grid(value.field_len(), ui, id, |ui, label| {
        (0..value.field_len()).for_each(|i| {
            if label {
                ui.label(i.to_string());
            }
            let field = value.field_mut(i).unwrap();
            let changed = ui_for_reflect(type_registry, field, ui, id.with(i));
            ui.end_row();
            changed
        });
    })
}

fn ui_for_struct(
    type_registry: &TypeRegistry,
    value: &mut dyn Struct,
    ui: &mut egui::Ui,
    id: egui::Id,
) {
    for i in 0..value.field_len() {
        CollapsingHeader::new(value.name_at(i).unwrap())
            .default_open(true)
            .id_source(i)
            .show(ui, |ui| {
                let field = value.field_at_mut(i).unwrap();
                ui_for_reflect(&type_registry, field, ui, id.with(i));
            });
    }
}

fn add<T: 'static>(type_registry: &mut TypeRegistry, fn_mut: GuiElementMutFn) {
    type_registry
        .get_mut(TypeId::of::<T>())
        .unwrap_or_else(|| panic!("Type {:?} not registered", type_name::<T>()))
        .insert(GuiElement { fn_mut });
}

fn initialize(mut reflection: ResMut<Reflection>) {
    let tr = &mut reflection.type_registry;

    add::<String>(tr, primitives::string_ui);
    add::<Cow<str>>(tr, primitives::cow_str_ui);

    add::<f32>(tr, primitives::number_ui::<f32>);
    add::<f64>(tr, primitives::number_ui::<f64>);
    add::<i8>(tr, primitives::number_ui::<i8>);
    add::<i16>(tr, primitives::number_ui::<i16>);
    add::<i32>(tr, primitives::number_ui::<i32>);
    add::<i64>(tr, primitives::number_ui::<i64>);
    add::<isize>(tr, primitives::number_ui::<isize>);
    add::<u8>(tr, primitives::number_ui::<u8>);
    add::<u16>(tr, primitives::number_ui::<u16>);
    add::<u32>(tr, primitives::number_ui::<u32>);
    add::<u64>(tr, primitives::number_ui::<u64>);
    add::<usize>(tr, primitives::number_ui::<usize>);
    add::<[[f32; 2]; 4]>(tr, primitives::quad_ui);
    add::<[f32; 2]>(tr, primitives::num_vector_ui::<f32, 2>);
    add::<[f32; 3]>(tr, primitives::num_vector_ui::<f32, 3>);
    add::<[u32; 2]>(tr, primitives::num_vector_ui::<u32, 2>);
    add::<[u32; 3]>(tr, primitives::num_vector_ui::<u32, 3>);
}

pub fn plugin(realm: &mut Realm) {
    realm
        .add_plugin(yapgeir_reflection::plugin)
        .register_type::<PathBuf>()
        .register_type::<OsString>()
        .register_type::<Option<String>>()
        .register_type::<Option<bool>>()
        .register_type::<Option<f64>>()
        .register_type::<Cow<'static, str>>()
        .register_type::<Cow<'static, Path>>()
        .register_type::<Duration>()
        .register_type::<Instant>()
        .register_type::<[[f32; 2]; 4]>()
        .register_type::<[f32; 2]>()
        .register_type::<[f32; 3]>()
        .register_type::<[u32; 2]>()
        .register_type::<[u32; 3]>()
        .run_system(initialize);
}

use eframe::egui;
use serde_json;
use serde_json::Value;
use crate::{INDENT_SPACES, UI_SPACE};

pub enum EditableValueRef<'a> {
    Json(&'a mut Value),
}


pub(crate) fn render_json_value_with_tracking(
    ui: &mut egui::Ui,
    value: &mut Value,
    modified: &mut bool,
    scroll_marker_key: &mut Option<String>,
    _content: &str,
    key_path: Vec<String>,
) {
    match value {
        Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                let mut new_path = key_path.clone();
                new_path.push(key.clone());
                let full_key = new_path.join(".");

                ui.horizontal(|ui| {
                    ui.add_space(INDENT_SPACES);
                    match v {
                        Value::Object(_) | Value::Array(_) => {
                            egui::CollapsingHeader::new(key)
                                .default_open(false)
                                .show(ui, |ui| {
                                    render_json_value_with_tracking(
                                        ui, v, modified, scroll_marker_key, _content, new_path,
                                    );
                                });
                        }
                        _ => {
                            ui.label(format!("{}:", key));
                            render_editable_value(
                                ui,
                                EditableValueRef::Json(v),
                                &full_key,
                                modified,
                                scroll_marker_key,
                            );
                        }
                    }
                });
            }
        }

        Value::Array(arr) => {
            render_array(ui, arr, key_path, modified, scroll_marker_key, _content);
        }

        _ => {
            render_editable_value(
                ui,
                EditableValueRef::Json(value),
                &key_path.join("."),
                modified,
                scroll_marker_key,
            );
        }
    }
}

fn render_editable_value(
    ui: &mut egui::Ui,
    value: EditableValueRef,
    key: &str,
    modified: &mut bool,
    scroll_marker_key: &mut Option<String>,
) {
    match value {
        EditableValueRef::Json(val) => match val {
            Value::String(s) => {
                let mut temp = s.clone();
                if ui.add(egui::TextEdit::singleline(&mut temp)).changed() {
                    *val = Value::String(temp);
                    *modified = true;
                    *scroll_marker_key = Some(key.to_string());
                }
            }
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    let mut temp = f;
                    if ui.add(egui::DragValue::new(&mut temp)).changed() {
                        *val = Value::from(temp);
                        *modified = true;
                        *scroll_marker_key = Some(key.to_string());
                    }
                }
            }
            Value::Bool(b) => {
                let mut temp = *b;
                if ui.checkbox(&mut temp, "").changed() {
                    *val = Value::Bool(temp);
                    *modified = true;
                    *scroll_marker_key = Some(key.to_string());
                }
            }
            Value::Null => {
                ui.label("null");
            }
            _ => {
                ui.label("(unsupported type)");
            }
        },
    }
}

fn render_array(
    ui: &mut egui::Ui,
    arr: &mut Vec<Value>,
    key_path: Vec<String>,
    modified: &mut bool,
    scroll_marker_key: &mut Option<String>,
    content: &str,
) {
    let mut to_remove = None;
    for (i, elem) in arr.iter_mut().enumerate() {
        let mut path = key_path.clone();
        path.push(i.to_string());

        ui.horizontal(|ui| {
            ui.add_space(INDENT_SPACES);
            render_json_value_with_tracking(ui, elem, modified, scroll_marker_key, content, path);
            if ui.button("\u{274C}").on_hover_text("Remove").clicked() {
                to_remove = Some(i);
            }
        });
    }

    if let Some(i) = to_remove {
        arr.remove(i);
        *modified = true;
    }

    ui.add_space(UI_SPACE);
    if ui.button("+ Add element").clicked() {
        arr.push(Value::Null);
        *modified = true;
    }
}

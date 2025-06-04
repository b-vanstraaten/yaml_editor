use crate::toml::Value;
use eframe::egui;
use crate::{INDENT_SPACES, UI_SPACE};

pub(crate) fn render_toml_value_with_tracking(
    ui: &mut egui::Ui,
    value: &mut Value,
    modified: &mut bool,
    scroll_marker_key: &mut Option<String>,
    content: &str,
    key_path: Vec<String>,
) {
    match value {
        Value::Table(table) => {
            for (key, v) in table.iter_mut() {
                let mut new_path = key_path.clone();
                new_path.push(key.to_string());
                let full_key = key.to_string();

                ui.horizontal(|ui| {
                    ui.add_space(INDENT_SPACES);
                    match v {
                        Value::Table(_) | Value::Array(_) => {
                            egui::CollapsingHeader::new(&full_key)
                                .default_open(false)
                                .show(ui, |ui| {
                                    render_toml_value_with_tracking(
                                        ui, v, modified, scroll_marker_key, content, new_path,
                                    );
                                });
                        }
                        _ => {
                            ui.label(format!("{}:", full_key));
                            render_editable_toml_value(ui, v, &full_key, modified, scroll_marker_key);
                        }
                    }
                });
            }
        }
        Value::Array(arr) => {
            for (idx, v) in arr.iter_mut().enumerate() {
                let mut new_path = key_path.clone();
                new_path.push(idx.to_string());
                let full_key = idx.to_string();

                ui.horizontal(|ui| {
                    ui.add_space(INDENT_SPACES);
                    match v {
                        Value::Table(_) | Value::Array(_) => {
                            egui::CollapsingHeader::new(&full_key)
                                .default_open(false)
                                .show(ui, |ui| {
                                    render_toml_value_with_tracking(
                                        ui, v, modified, scroll_marker_key, content, new_path,
                                    );
                                });
                        }
                        _ => {
                            ui.label(format!("{}:", full_key));
                            render_editable_toml_value(ui, v, &full_key, modified, scroll_marker_key);
                        }
                    }
                });
            }
        }
        _ => {}
    }
}

fn render_editable_toml_value(
    ui: &mut egui::Ui,
    value: &mut Value,
    key: &str,
    modified: &mut bool,
    scroll_marker_key: &mut Option<String>,
) {
    match value {
        Value::String(s) => {
            let mut val = s.clone();
            if ui.add(egui::TextEdit::singleline(&mut val)).changed() {
                *value = Value::String(val);
                *modified = true;
                *scroll_marker_key = Some(key.to_string());
            }
        }
        Value::Integer(n) => {
            let mut val = *n;
            if ui.add(egui::DragValue::new(&mut val)).changed() {
                *value = Value::Integer(val);
                *modified = true;
                *scroll_marker_key = Some(key.to_string());
            }
        }
        Value::Boolean(b) => {
            let mut state = *b;
            if ui.checkbox(&mut state, "").changed() {
                *value = Value::Boolean(state);
                *modified = true;
                *scroll_marker_key = Some(key.to_string());
            }
        }
        Value::Float(f) => {
            let mut val = *f;
            if ui.add(egui::DragValue::new(&mut val)).changed() {
                *value = Value::Float(val);
                *modified = true;
                *scroll_marker_key = Some(key.to_string());
            }
        }
        _ => {}
    }
}
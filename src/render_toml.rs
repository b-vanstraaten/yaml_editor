use crate::toml::Value;
use eframe::egui;
use crate::{INDENT_SPACES};

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
            // Mutable statics for new key/val input fields
            static mut NEW_KEY: String = String::new();
            static mut NEW_VAL: String = String::new();
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
            // Section for adding a new key-value pair
            ui.separator();
            ui.horizontal(|ui| {
                unsafe {
                    ui.colored_label(egui::Color32::GRAY, "New Key:");
                    let key_changed = ui.text_edit_singleline(&mut NEW_KEY).changed();
                    ui.colored_label(egui::Color32::GRAY, "Value:");
                    let val_changed = ui.text_edit_singleline(&mut NEW_VAL).changed();
                    if key_changed || val_changed {
                        if !NEW_KEY.trim().is_empty() && !table.contains_key(&NEW_KEY) {
                            if let Some(parsed_val) = value_from_string(&NEW_VAL) {
                                table.insert(NEW_KEY.clone(), parsed_val);
                                *modified = true;
                                *scroll_marker_key = Some(NEW_KEY.clone());
                                NEW_KEY.clear();
                                NEW_VAL.clear();
                            }
                        }
                    }
                }
            });
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
/// Unescapes TOML-style escape sequences in a string, such as `\n`, `\t`, `\\`, `\"`, and unicode escapes like `\uXXXX`.
fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('u') => {
                    let unicode: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u16::from_str_radix(&unicode, 16) {
                        if let Some(ch) = std::char::from_u32(code as u32) {
                            result.push(ch);
                        }
                    }
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn value_from_string(input: &str) -> Option<Value> {
    if input.eq_ignore_ascii_case("true") {
        return Some(Value::Boolean(true));
    }
    if input.eq_ignore_ascii_case("false") {
        return Some(Value::Boolean(false));
    }
    if let Ok(int_val) = input.parse::<i64>() {
        return Some(Value::Integer(int_val));
    }
    if let Ok(float_val) = input.parse::<f64>() {
        return Some(Value::Float(float_val));
    }
    let stripped = input.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(input);
    Some(Value::String(stripped.to_string()))
}
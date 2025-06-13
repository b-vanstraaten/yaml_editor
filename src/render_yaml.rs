use eframe::egui;
use crate::{INDENT_SPACES};
use yaml_rust::Yaml;

pub(crate) fn render_yaml_value_with_tracking(
    ui: &mut egui::Ui,
    value: &mut Yaml,
    modified: &mut bool,
    scroll_marker_key: &mut Option<String>,
    content: &str,
    key_path: Vec<String>,
) {
    match value {
        Yaml::Hash(map) => {
            for (k, v) in &mut *map {
                if let Yaml::String(key_str) = k {
                    let mut new_path = key_path.clone();
                    new_path.push(key_str.clone());
                    let full_key = key_str;

                    ui.horizontal(|ui| {
                        ui.add_space(INDENT_SPACES);
                        match v {
                            Yaml::Hash(_) | Yaml::Array(_) => {
                                egui::CollapsingHeader::new(full_key)
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        render_yaml_value_with_tracking(
                                            ui, v, modified, scroll_marker_key, content, new_path,
                                        );
                                    });
                            }
                            _ => {
                                ui.label(format!("{}:", full_key));
                                render_editable_yaml_value(
                                    ui, v, full_key, modified, scroll_marker_key,
                                );
                            }
                        }
                    });
                }
            }

            use egui::TextEdit;

            let key_id = egui::Id::new("new_key_input").with(ui.id());
            let value_id = egui::Id::new("new_value_input").with(ui.id());

            egui::CollapsingHeader::new("Add New Field")
                .default_open(false)
                .show(ui, |ui| {
                    let mut key_input = ui
                        .memory_mut(|mem| mem.data.get_temp::<String>(key_id))
                        .unwrap_or_default();
                    let mut value_input = ui
                        .memory_mut(|mem| mem.data.get_temp::<String>(value_id))
                        .unwrap_or_default();

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Key:");
                        let key_response = ui.add(TextEdit::singleline(&mut key_input).hint_text("key").desired_width(100.0));
                        ui.label("Value:");
                        let value_response = ui.add(TextEdit::singleline(&mut value_input).hint_text("value").desired_width(100.0));

                        if (key_response.lost_focus() || value_response.lost_focus())
                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            if !key_input.trim().is_empty() {
                                let inferred_value = if value_input.eq_ignore_ascii_case("true") {
                                    Yaml::Boolean(true)
                                } else if value_input.eq_ignore_ascii_case("false") {
                                    Yaml::Boolean(false)
                                } else if value_input.eq_ignore_ascii_case("null") {
                                    Yaml::Null
                                } else if let Ok(i) = value_input.parse::<i64>() {
                                    Yaml::Integer(i)
                                } else if let Ok(f) = value_input.parse::<f64>() {
                                    Yaml::Real(f.to_string())
                                } else {
                                    Yaml::String(value_input.clone())
                                };

                                map.insert(Yaml::String(key_input.clone()), inferred_value);
                                *modified = true;
                                *scroll_marker_key = Some(key_input.clone());
                                key_input.clear();
                                value_input.clear();
                            }
                        }
                    });

                    ui.memory_mut(|mem| {
                        mem.data.insert_temp(key_id, key_input);
                        mem.data.insert_temp(value_id, value_input);
                    });
                });
        }
        _ => {}
    }
}

fn render_editable_yaml_value(
    ui: &mut egui::Ui,
    value: &mut Yaml,
    key: &str,
    modified: &mut bool,
    scroll_marker_key: &mut Option<String>,
) {
    match value {
        Yaml::String(s) => {
            let mut val = s.clone();
            if ui.add(egui::TextEdit::singleline(&mut val)).changed() {
                *value = Yaml::String(val);
                *modified = true;
                *scroll_marker_key = Some(key.to_string());
            }
        }
        Yaml::Real(s) => {
            if let Ok(f) = s.parse::<f64>() {
                let mut val = f;
                if ui.add(egui::DragValue::new(&mut val)).changed() {
                    *value = Yaml::Real(val.to_string());
                    *modified = true;
                    *scroll_marker_key = Some(key.to_string());
                }
            }
        }
        Yaml::Integer(i) => {
            let mut val = *i;
            if ui.add(egui::DragValue::new(&mut val)).changed() {
                *value = Yaml::Integer(val);
                *modified = true;
                *scroll_marker_key = Some(key.to_string());
            }
        }
        Yaml::Boolean(b) => {
            let mut state = *b;
            if ui.checkbox(&mut state, "").changed() {
                *value = Yaml::Boolean(state);
                *modified = true;
                *scroll_marker_key = Some(key.to_string());
            }
        }
        Yaml::Null => {
            let mut input = String::new();
            if ui.add(egui::TextEdit::singleline(&mut input).hint_text("null")).changed() {
                // Try to infer type
                if input.eq_ignore_ascii_case("true") {
                    *value = Yaml::Boolean(true);
                } else if input.eq_ignore_ascii_case("false") {
                    *value = Yaml::Boolean(false);
                } else if let Ok(i) = input.parse::<i64>() {
                    *value = Yaml::Integer(i);
                } else if let Ok(f) = input.parse::<f64>() {
                    *value = Yaml::Real(f.to_string());
                } else {
                    *value = Yaml::String(input.clone());
                }
                *modified = true;
                *scroll_marker_key = Some(key.to_string());
            }
        }
        _ => {
            ui.label(format!("{:?}", value)); // fallback for other types
        }
    }
}
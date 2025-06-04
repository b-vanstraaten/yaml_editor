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
            for (k, v) in map {
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
        _ => {
            ui.label(format!("{:?}", value)); // fallback for other types
        }
    }
}
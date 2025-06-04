use eframe::egui;
use crate::{INDENT_SPACES, UI_SPACE};

pub(crate) fn render_yaml_value_with_tracking(
    ui: &mut egui::Ui,
    value: &mut serde_yaml::Value,
    modified: &mut bool,
    scroll_marker_key: &mut Option<String>,
    content: &str,
    key_path: Vec<String>,
) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map.iter_mut() {
                if let serde_yaml::Value::String(key_str) = k {
                    let mut new_path = key_path.clone();
                    new_path.push(key_str.clone());
                    let full_key = key_str;

                    ui.horizontal(|ui| {
                        ui.add_space(INDENT_SPACES);
                        match v {
                            serde_yaml::Value::Mapping(_) | serde_yaml::Value::Sequence(_) => {
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
    value: &mut serde_yaml::Value,
    key: &str,
    modified: &mut bool,
    scroll_marker_key: &mut Option<String>,
) {
    match value {
        serde_yaml::Value::String(s) => {
            let mut val = s.clone(); // no quotes in UI
            if ui.add(egui::TextEdit::singleline(&mut val)).changed() {
                *value = serde_yaml::Value::String(val);
                *modified = true;
                *scroll_marker_key = Some(key.to_string());
            }
        }
        serde_yaml::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                let mut val = f;
                if ui.add(egui::DragValue::new(&mut val)).changed() {
                    *value = serde_yaml::from_str(&val.to_string())
                        .unwrap_or(serde_yaml::Value::Null);
                    *modified = true;
                    *scroll_marker_key = Some(key.to_string());
                }
            }
        }
        serde_yaml::Value::Bool(b) => {
            let mut state = *b;
            if ui.checkbox(&mut state, "").changed() {
                *value = serde_yaml::Value::Bool(state);
                *modified = true;
                *scroll_marker_key = Some(key.to_string());
            }
        }
        _ => {
            ui.label(format!("{:?}", value)); // fallback for other types
        }
    }
}
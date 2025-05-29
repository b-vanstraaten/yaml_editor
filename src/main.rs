// Full Rust YAML Editor with persistent file selection and GUI file picker
use std::{
    fs,
    sync::{Arc, Mutex},
};

use eframe::{egui, App, Frame};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde_yaml::Value;
use tokio::sync::mpsc;
use rfd::FileDialog;
use directories::ProjectDirs;


const WINDOW_HEIGHT: f32 = 1000.;
const WINDOW_WIDTH: f32 = 500.;

const UI_SPACE: f32 = 2.;
const INDENT_SPACES: f32 = 24.;
const RAW_EDITOR_WIDTH_FRACTION: f32 = 0.5;
const CONFIG_FILE_NAME: &str = "last_opened_file.txt";

fn get_config_file_path() -> Option<std::path::PathBuf> {
    ProjectDirs::from("org", "QuantumTools", "YamlEditor").map(|proj_dirs| {
        let dir = proj_dirs.config_dir();
        // Attempt to create the directory, ignore error if it fails
        let _ = fs::create_dir_all(dir);
        dir.join(CONFIG_FILE_NAME)
    })
}

fn load_saved_file_path() -> Option<String> {
    get_config_file_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| std::path::Path::new(s).exists())
}

fn save_file_path(path: &str) {
    if let Some(config_path) = get_config_file_path() {
        let _ = fs::write(config_path, path);
    }
}

struct YamlEditorApp {
    content: Arc<Mutex<String>>,
    file_path: Arc<Mutex<String>>,
    show_raw_editor: bool,
    dark_mode: bool,
    scroll_marker_key: Option<String>,
    search_query: String,
    search_triggered: bool,
}

impl YamlEditorApp {
    fn new(file_path: Arc<Mutex<String>>, content: Arc<Mutex<String>>) -> Self {
        Self {
            content,
            file_path,
            show_raw_editor: false,
            dark_mode: true,
            scroll_marker_key: None,
            search_query: String::new(),
            search_triggered: false,
        }
    }

    fn render_toolbar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical(|ui| {

            // Line 1: Buttons and checkboxes
            ui.horizontal(|ui| {
                if ui.button("📂 Change File").clicked() {
                    if let Some(path_buf) = FileDialog::new().add_filter("YAML", &["yaml", "yml"]).pick_file() {
                        if let Ok(new_path) = path_buf.into_os_string().into_string() {
                            let mut file_path = self.file_path.lock().unwrap();
                            let mut content = self.content.lock().unwrap();
                            *file_path = new_path.clone();
                            *content = load_file(&new_path);
                            save_file_path(&new_path);
                            ctx.request_repaint();
                        }
                    }
                }
                ui.checkbox(&mut self.show_raw_editor, "📝 Show Raw Editor");
                ui.checkbox(&mut self.dark_mode, "🌗 Dark Mode");
            });

            // Line 2: File label and path
            ui.horizontal(|ui| {
                ui.label("📁 File:");
                let path = self.file_path.lock().unwrap();
                ui.label(egui::RichText::new(path.as_str()).monospace());
            });

        });
    }

    fn render_editors(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, content: &mut String) {
        let total_height = ui.available_height();
        let total_width = ui.available_width();

        ui.allocate_ui_with_layout(
            egui::Vec2::new(total_width, total_height),
            egui::Layout::left_to_right(egui::Align::Min),
            |ui| {
                if self.show_raw_editor {
                    self.render_raw_editor(ui, ctx, content, total_width, total_height);
                    ui.separator();
                }
                self.render_collapsible_view(ui, content, total_width, total_height);
            },
        );
    }

    fn render_raw_editor(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, content: &mut String, width: f32, height: f32) {
        ui.allocate_ui_with_layout(
            egui::Vec2::new(width * RAW_EDITOR_WIDTH_FRACTION, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.label("📝 Raw YAML Editor:");
                ui.horizontal(|ui| {
                    ui.label("🔍 Search:");
                    let search_input = ui.text_edit_singleline(&mut self.search_query);
                    if search_input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.scroll_marker_key = Some(self.search_query.clone());
                        self.search_triggered = true;
                        ctx.request_repaint();
                    }
                });

                let text_edit_id = ui.make_persistent_id("raw_editor_text");
                let scroll_area_id = egui::Id::new("raw_editor_scroll");

                // Clone content for search operations to avoid borrowing issues
                let content_for_search = content.clone();

                // Calculate scroll offset if search was triggered
                let mut target_scroll_offset = None;
                if self.search_triggered {
                    if let Some(search_text) = &self.scroll_marker_key {
                        if let Some(pos) = content_for_search.to_lowercase().find(&search_text.to_lowercase()) {
                            let font_id = egui::TextStyle::Monospace.resolve(&ctx.style());
                            let row_height = ctx.fonts(|f| f.row_height(&font_id));
                            let preceding_text = &content_for_search[..pos];
                            let line_number = preceding_text.chars().filter(|&c| c == '\n').count();
                            let target_y = line_number as f32 * row_height;

                            // Set target scroll offset to center the line
                            target_scroll_offset = Some(target_y - height * 0.0);
                        }
                    }
                    self.search_triggered = false;
                }

                // Create scroll area with potential offset
                let mut scroll_area = egui::ScrollArea::vertical()
                    .id_salt(scroll_area_id)
                    .auto_shrink([false; 2]);

                // Apply scroll offset if we have a target
                if let Some(offset) = target_scroll_offset {
                    scroll_area = scroll_area.vertical_scroll_offset(offset.max(0.0));
                }

                scroll_area.show(ui, |ui| {
                    let text_edit = egui::TextEdit::multiline(content)
                        .id(text_edit_id)
                        .desired_width(width * RAW_EDITOR_WIDTH_FRACTION - 20.0)
                        .font(egui::TextStyle::Monospace);

                    let response = ui.add(text_edit);

                    if response.changed() {
                        if let Ok(_) = std::fs::write(&*self.file_path.lock().unwrap(), content) {
                            ctx.request_repaint();
                        }
                    }

                    // Focus the text editor if search was triggered
                    if target_scroll_offset.is_some() {
                        ctx.memory_mut(|mem| {
                            mem.request_focus(text_edit_id);
                        });
                    }

                    response
                });
            },
        );
    }


    fn render_collapsible_view(&mut self, ui: &mut egui::Ui, content: &mut String, width: f32, height: f32) {
        ui.allocate_ui_with_layout(
            egui::Vec2::new(width * if self.show_raw_editor { 1. - RAW_EDITOR_WIDTH_FRACTION } else { 1.0 }, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.label("📂 Collapsible YAML View:");
                egui::ScrollArea::vertical()
                    .id_salt("collapsible_yaml_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            match serde_yaml::from_str::<Value>(&*content) {
                                Ok(mut parsed) => {
                                    let mut modified = false;
                                    render_yaml_value_with_tracking(
                                        ui,
                                        &mut parsed,
                                        &mut modified,
                                        &mut self.scroll_marker_key,
                                        &content,
                                        vec![]
                                    );
                                    ui.add_space(20.0);

                                    if modified {
                                        if let Ok(updated) = serde_yaml::to_string(&parsed) {
                                            *content = updated;
                                            let _ = fs::write(&*self.file_path.lock().unwrap(), &*content);
                                        }
                                    }
                                }
                                Err(err) => {
                                    ui.colored_label(
                                        egui::Color32::RED,
                                        format!("⚠️ Invalid YAML: {err}"),
                                    );
                                }
                            }
                        });
                    });
            },
        );
    }
}

impl App for YamlEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        let mut content_owned = self.content.lock().unwrap().clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_toolbar(ui, ctx);
            ui.separator();
            self.render_editors(ui, ctx, &mut content_owned);
        });

        let mut content_guard = self.content.lock().unwrap();
        if *content_guard != content_owned {
            *content_guard = content_owned;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let file_path = load_saved_file_path().or_else(|| {
        FileDialog::new()
            .add_filter("YAML", &["yaml", "yml"])
            .pick_file()
            .and_then(|p| p.into_os_string().into_string().ok())
    }).unwrap_or_else(|| std::process::exit(0));

    save_file_path(&file_path);

    let (file_path, content) = init_file_state(&file_path);
    let (tx, rx) = mpsc::channel(100);

    init_file_watcher(tx, &file_path);
    spawn_file_watcher(rx, Arc::clone(&file_path), Arc::clone(&content));

    eframe::run_native(
        "YAML Editor",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
            ..Default::default()
        },
        Box::new(|_cc| {
            Ok(Box::new(YamlEditorApp::new(file_path, content)) as Box<dyn App>)
        })
    )
}

fn init_file_state(path: &str) -> (Arc<Mutex<String>>, Arc<Mutex<String>>) {
    let file_path = Arc::new(Mutex::new(path.to_string()));
    let content = Arc::new(Mutex::new(load_file(path)));
    (file_path, content)
}

fn load_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| String::new())
}

fn init_file_watcher(tx: mpsc::Sender<Event>, file_path: &Arc<Mutex<String>>) {
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, _>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        },
        Config::default(),
    ).unwrap();

    watcher.watch(file_path.lock().unwrap().as_ref(), RecursiveMode::NonRecursive).unwrap();
}

fn spawn_file_watcher(mut rx: mpsc::Receiver<Event>, file_path: Arc<Mutex<String>>, content: Arc<Mutex<String>>) {
    tokio::spawn(async move {
        while let Some(_) = rx.recv().await {
            if let Ok(new_content) = fs::read_to_string(&*file_path.lock().unwrap()) {
                let mut lock = content.lock().unwrap();
                if *lock != new_content {
                    *lock = new_content;
                }
            }
        }
    });
}

fn render_yaml_value_with_tracking(
    ui: &mut egui::Ui,
    value: &mut Value,
    modified: &mut bool,
    scroll_marker_key: &mut Option<String>,
    content: &str,
    key_path: Vec<String>,
) {
    match value {
        Value::Mapping(map) => {
            for (k, v) in map.iter_mut() {
                if let Value::String(key_str) = k {
                    let mut new_path = key_path.clone();
                    new_path.push(key_str.clone());
                    let full_key = key_str;

                    match v {
                        Value::Mapping(_) | Value::Sequence(_) => {
                            ui.add_space(UI_SPACE);
                            ui.horizontal(|ui| {
                                ui.add_space(INDENT_SPACES);
                                egui::CollapsingHeader::new(full_key)
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        render_yaml_value_with_tracking(
                                            ui,
                                            v,
                                            modified,
                                            scroll_marker_key,
                                            content,
                                            new_path,
                                        );
                                    });
                            });
                        }
                        Value::String(s) => {
                            ui.horizontal(|ui| {
                                ui.add_space(INDENT_SPACES);
                                ui.label(format!("{}:", full_key));
                                let response = ui.add(egui::TextEdit::singleline(s));
                                if response.changed() {
                                    *modified = true;
                                    *scroll_marker_key = Some(full_key.to_string());
                                }
                            });
                        }
                        Value::Number(n) => {
                            if let Some(f) = n.as_f64() {
                                let mut val = f;
                                ui.horizontal(|ui| {
                                    ui.add_space(INDENT_SPACES);
                                    ui.label(format!("{}:", full_key));
                                    if ui.add(egui::DragValue::new(&mut val)).changed() {
                                        *v = serde_yaml::from_str(&val.to_string()).unwrap_or(Value::Null);
                                        *modified = true;
                                        *scroll_marker_key = Some(full_key.to_string());
                                    }
                                });
                            }
                        }
                        Value::Bool(b) => {
                            let mut state = *b;
                            ui.horizontal(|ui| {
                                ui.add_space(INDENT_SPACES);
                                ui.label(format!("{}:", full_key));
                                if ui.checkbox(&mut state, "").changed() {
                                    *v = Value::Bool(state);
                                    *modified = true;
                                    *scroll_marker_key = Some(full_key.to_string());
                                }
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {}
    }
}

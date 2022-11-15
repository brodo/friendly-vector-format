use eframe::egui;
use tokio::sync::watch::Receiver;
use tree_sitter::Tree;

#[derive(Debug, Clone)]
pub struct AppUpdate {
    pub tree: Tree,
    pub code: String,
}

pub struct MyApp {
    update_receiver: Receiver<Result<AppUpdate, String>>,
}

impl MyApp {
    pub fn new_with_update_receiver(receiver: Receiver<Result<AppUpdate, String>>) -> Self {
        MyApp {
            update_receiver: receiver,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(has_changed) = self.update_receiver.has_changed() {
            if has_changed {
                ctx.request_repaint();
            }
        }
        let Ok(app_update) = self.update_receiver.borrow().clone() else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("There is a problem updating from the LSP");
            });
            return;
        };


        egui::SidePanel::left("inspector_panel").show(ctx, |ui| {
            ui.label("Hello World!");
        });
        egui::TopBottomPanel::bottom("source_code_panel").show(ctx, |ui| {
            ui.label("Code:");
            let mut code = app_update.code;
            ui.text_edit_multiline(&mut code);
            ui.label("Tree:");
            let mut tree_str = format!("{:?}", app_update.tree);
            ui.text_edit_multiline(&mut tree_str);
        });
    }
}

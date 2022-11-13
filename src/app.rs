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
    age: usize
}

impl MyApp {
    pub fn new_with_update_receiver(receiver: Receiver<Result<AppUpdate, String>>) -> Self {
        MyApp {
            update_receiver: receiver,
            age: 10
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Friendly Vector Format Live Preview");
            if let Ok(has_changed) = self.update_receiver.has_changed() {
                if has_changed {
                    ctx.request_repaint();
                }
            }
            match self.update_receiver.borrow().clone() {
                Ok(parsed_code) => {
                    ui.label("Code:");
                    let mut code = parsed_code.code;
                    ui.text_edit_multiline(&mut code);
                    ui.label("Tree:");
                    let mut tree_str = format!("{:?}", parsed_code.tree);
                    ui.text_edit_multiline(&mut tree_str);
                }
                Err(e) => {
                    ui.label(e);
                }
            }

            // let text = self.ast_receiver.borrow().to_string();
            // ui.label(text);
            // ui.horizontal(|ui| {
            //     ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name);
            // });
            // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("age: {}", self.age));
        });
    }
}


use eframe::egui;
use tokio::sync::watch::Receiver;

pub struct MyApp {
    ast_receiver: Receiver<String>,
}

impl MyApp {
    pub fn new_with_ast_receiver(receiver: Receiver<String>) -> Self{
        MyApp {
            ast_receiver: receiver,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {


        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Friendly Vector Format Live Preview");
            let text = self.ast_receiver.borrow().to_string();
            ui.label(text);
            // ui.horizontal(|ui| {
            //     ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name);
            // });
            // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            // if ui.button("Click each year").clicked() {
            //     self.age += 1;
            // }
            // ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}
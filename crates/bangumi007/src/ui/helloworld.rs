#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use eframe::egui::ecolor;
use crate::module::core::init::run_init;
use crate::module::logger;

pub(crate) fn ui_main() -> Result<(), eframe::Error> {
    run_init().unwrap();
    let ui_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Bangumi007",
        ui_options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(MyApp::new(cc))
        }),
    )
}

struct MyApp {
    name: String,
    age: u32,
    input_text: String,
}

fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("sourcehansans_sc".to_owned(),
                           egui::FontData::from_static(include_bytes!("../../../../assets/fonts/SourceHanSans/SC/SourceHanSansSC-Normal.otf")));
    fonts.font_data.insert("sourcehansans_tc".to_owned(),
                           egui::FontData::from_static(include_bytes!("../../../../assets/fonts/SourceHanSans/TC/SourceHanSansTC-Normal.otf")));
    fonts.font_data.insert("sourcehansans_j".to_owned(),
                           egui::FontData::from_static(include_bytes!("../../../../assets/fonts/SourceHanSans/J/SourceHanSans-Normal.otf")));
    fonts.font_data.insert("cascadia_code".to_owned(),
                           egui::FontData::from_static(include_bytes!("../../../../assets/fonts/CascadiaCode.ttf")));
    // fonts.font_data.get_mut("source_han_hans_sc_vf").unwrap().index = 3;
    fonts.families.entry(egui::FontFamily::Proportional).or_default()
        .insert(0, "sourcehansans_sc".to_owned());
    fonts.families.entry(egui::FontFamily::Proportional).or_default()
        .insert(1, "sourcehansans_tc".to_owned());
    fonts.families.entry(egui::FontFamily::Proportional).or_default()
        .insert(2, "sourcehansans_j".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default()
        .insert(0, "cascadia_code".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default()
        .insert(1, "sourcehansans_sc".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default()
        .insert(2, "sourcehansans_tc".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default()
        .insert(3, "sourcehansans_j".to_owned());
    ctx.set_fonts(fonts);
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_fonts(&cc.egui_ctx);
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(ecolor::Color32::from_rgba_premultiplied(220, 220, 220, 255));
        &cc.egui_ctx.set_visuals(visuals);
        Self::default()
    }
}


impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            input_text: "Default text".to_owned(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(2.0);
            ui.heading("Bangumi007");
            ui.add_space(2.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Hello World!");
                    ui.label("This is a simple egui app.");
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.label("Hello World!");
                    ui.label("This is a simple egui app.");
                });
            });
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));

            ui.image(egui::include_image!(
                "../../../../assets/ferris.png"
            ));

            ui.text_edit_multiline(&mut self.input_text);

        });
    }
}
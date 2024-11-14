#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use colog;
use eframe::egui::{
    self, ColorImage, Image, TextureFilter, TextureHandle, TextureOptions, ThemePreference,
};
use log::error;
use luca_pak::{entry::EntryType, Pak};
use std::fs;

fn main() -> eframe::Result {
    colog::default_builder()
        .filter(None, log::LevelFilter::Warn)
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "LUCA PAK Explorer",
        options,
        Box::new(|ctx| {
            let ppp = ctx.egui_ctx.pixels_per_point() * 1.5;
            ctx.egui_ctx.set_pixels_per_point(ppp);
            Ok(Box::<PakExplorer>::default())
        }),
    )
}

struct PakExplorer {
    open_file: Option<Pak>,
    selected_entry: Option<luca_pak::entry::Entry>,
    image_texture: Option<egui::TextureHandle>,
    hex_string: Option<Vec<String>>,
}

impl Default for PakExplorer {
    fn default() -> Self {
        Self {
            open_file: None,
            selected_entry: None,
            image_texture: None,
            hex_string: None,
        }
    }
}

impl eframe::App for PakExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PAK File Explorer");
            ctx.options_mut(|o| o.theme_preference = ThemePreference::System);

            ui.horizontal(|ui| {
                if ui.button("Open file").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        let pak = match Pak::open(&path) {
                            Ok(pak) => Some(pak),
                            Err(e) => {
                                error!("Unable to read selected file as PAK: {}", e);
                                None
                            }
                        };
                        self.open_file = pak;
                        self.selected_entry = None;
                        self.image_texture = None;
                        self.hex_string = None;
                    }
                }
                if let Some(pak) = &self.open_file {
                    if ui.button("Save PAK").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name(pak.path().file_name().unwrap().to_string_lossy())
                            .save_file()
                        {
                            pak.save(&path).unwrap();
                        }
                    }
                }
            });

            ui.separator();

            if let Some(pak) = &self.open_file {
                ui.label(format!(
                    "Opened {}",
                    pak.path().file_name().unwrap().to_string_lossy()
                ));
                ui.label(format!("Contains {} Entries", pak.entries().len()));

                let selection = if let Some(entry) = &self.selected_entry {
                    entry.display_name()
                } else {
                    "None".to_string()
                };

                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_source("my-combobox")
                        .selected_text(selection.clone())
                        .truncate()
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_entry, None, "");
                            for entry in pak.entries() {
                                if ui
                                    .selectable_value(
                                        &mut self.selected_entry,
                                        Some(entry.clone()),
                                        format!("{} - {}", entry.display_name(), entry.id()),
                                    )
                                    .clicked()
                                {
                                    self.image_texture = None;
                                };
                            }
                        });

                    if let Some(entry) = &self.selected_entry {
                        ui.label(format!("Index {}, ID {}", entry.index(), entry.id()));
                    }
                });
            } else {
                ui.centered_and_justified(|ui| ui.label("No File Opened"));
            }

            if let Some(entry) = &self.selected_entry {
                ui.horizontal(|ui| {
                    if ui.button("Save entry").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name(entry.display_name())
                            .save_file()
                        {
                            entry.save(&path).unwrap();
                        }
                    }

                    if let Some(pak) = &mut self.open_file.as_mut() {
                        if ui.button("Replace entry").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                let file_bytes = fs::read(path).unwrap();
                                pak.replace(entry.index(), &file_bytes).unwrap();
                            }
                        }
                    }
                });
                match entry.file_type() {
                    EntryType::CZ0
                    | EntryType::CZ1
                    | EntryType::CZ2
                    | EntryType::CZ3
                    | EntryType::CZ4
                    | EntryType::CZ5 => {
                        if ui.button("Save as PNG").clicked() {
                            let mut display_name = entry.display_name();
                            display_name.push_str(".png");
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name(display_name)
                                .save_file()
                            {
                                let cz =
                                    cz::CzFile::decode(&mut std::io::Cursor::new(entry.as_bytes()))
                                        .unwrap();
                                image::save_buffer_with_format(
                                    path,
                                    cz.as_raw(),
                                    cz.header().width() as u32,
                                    cz.header().height() as u32,
                                    image::ColorType::Rgba8,
                                    image::ImageFormat::Png,
                                )
                                .unwrap();
                            }
                        }

                        ui.separator();

                        let texture: &TextureHandle = self.image_texture.get_or_insert_with(|| {
                            let cz =
                                cz::CzFile::decode(&mut std::io::Cursor::new(entry.as_bytes()))
                                    .unwrap();
                            let image = ColorImage::from_rgba_unmultiplied(
                                [cz.header().width() as usize, cz.header().height() as usize],
                                cz.as_raw(),
                            );
                            ui.ctx().load_texture(
                                "eventframe",
                                image,
                                TextureOptions {
                                    magnification: TextureFilter::Nearest,
                                    minification: TextureFilter::Linear,
                                    ..Default::default()
                                },
                            )
                        });

                        ui.centered_and_justified(|ui| {
                            ui.add(
                                Image::from_texture(texture)
                                    .show_loading_spinner(true)
                                    .shrink_to_fit()
                                    .rounding(2.0),
                            )
                        });
                    }
                    _ => {
                        ui.centered_and_justified(|ui| ui.label("No Preview Available"));
                    }
                }
            } else if self.open_file.is_some() {
                ui.centered_and_justified(|ui| ui.label("Select an Entry"));
            }
        });
    }
}

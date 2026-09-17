use chrono::{Datelike, Local, NaiveDate};
use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, RichText, Sense, Vec2, Visuals};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
struct AppData {
    worked_days: HashSet<NaiveDate>,
}

struct WorkCalendar {
    data: AppData,
    current_year: i32,
    current_month: u32,
    data_path: PathBuf,
    use_custom_theme: bool,
}

impl WorkCalendar {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = FontDefinitions::default();

        let font_paths = [
            "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf",
            "/usr/share/fonts/TTF/JetBrainsMonoNLNerdFont-Regular.ttf",
            "/usr/share/fonts/jetbrains-mono-nerd/JetBrainsMonoNerdFont-Regular.ttf",
            "/usr/share/fonts/OTF/JetBrainsMonoNerdFont-Regular.otf",
            "/usr/local/share/fonts/JetBrainsMonoNerdFont-Regular.ttf",
        ];

        for path in font_paths {
            if let Ok(font_data) = fs::read(path) {
                fonts.font_data.insert(
                    "JetBrainsMonoNerd".to_owned(),
                    FontData::from_owned(font_data),
                );

                fonts
                    .families
                    .entry(FontFamily::Proportional)
                    .or_default()
                    .insert(0, "JetBrainsMonoNerd".to_owned());

                fonts
                    .families
                    .entry(FontFamily::Monospace)
                    .or_default()
                    .insert(0, "JetBrainsMonoNerd".to_owned());

                break;
            }
        }

        cc.egui_ctx.set_fonts(fonts);

        let data_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("worked_days.json");

        let data = if data_path.exists() {
            let content = fs::read_to_string(&data_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppData::default()
        };

        let now = Local::now().date_naive();

        Self {
            data,
            current_year: now.year(),
            current_month: now.month(),
            data_path,
            use_custom_theme: true,
        }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&self.data_path, json);
        }
    }

    fn toggle_day(&mut self, date: NaiveDate) {
        if self.data.worked_days.contains(&date) {
            self.data.worked_days.remove(&date);
        } else {
            self.data.worked_days.insert(date);
        }
        self.save();
    }

    fn lain_theme() -> Visuals {
        let accent = Color32::from_rgb(172, 130, 233);
        let accent_deep = Color32::from_rgb(143, 86, 225);
        let dark = Color32::from_rgb(20, 18, 22);
        let lighter_dark = Color32::from_rgb(39, 35, 43);
        let foreground = Color32::from_rgb(216, 202, 184);
        let complementary = Color32::from_rgb(196, 232, 129);

        let mut visuals = Visuals::dark();

        visuals.dark_mode = true;
        visuals.override_text_color = Some(foreground);

        visuals.widgets.noninteractive.bg_fill = dark;
        visuals.widgets.inactive.bg_fill = lighter_dark;
        visuals.widgets.hovered.bg_fill = accent_deep;
        visuals.widgets.active.bg_fill = accent;
        visuals.widgets.open.bg_fill = accent_deep;

        visuals.widgets.noninteractive.fg_stroke.color = foreground;
        visuals.widgets.inactive.fg_stroke.color = foreground;
        visuals.widgets.hovered.fg_stroke.color = foreground;
        visuals.widgets.active.fg_stroke.color = Color32::WHITE;
        visuals.widgets.open.fg_stroke.color = Color32::WHITE;

        visuals.selection.bg_fill = accent.gamma_multiply(0.35);
        visuals.selection.stroke.color = accent;

        visuals.panel_fill = dark;
        visuals.window_fill = dark;
        visuals.extreme_bg_color = Color32::from_rgb(12, 10, 14);
        visuals.faint_bg_color = lighter_dark;

        visuals.hyperlink_color = complementary;
        visuals.warn_fg_color = complementary;

        visuals
    }
}

impl eframe::App for WorkCalendar {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.use_custom_theme {
            ctx.set_visuals(Self::lain_theme());
        } else {
            ctx.set_visuals(Visuals::dark());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.heading("Lainlog");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let theme_icon = if self.use_custom_theme {
                            "󰔎"
                        } else {
                            "󰔏"
                        };

                        if ui
                            .add_sized(
                                [28.0, 28.0],
                                egui::Button::new(RichText::new(theme_icon).size(16.0)),
                            )
                            .clicked()
                        {
                            self.use_custom_theme = !self.use_custom_theme;
                        }
                    });
                });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button(RichText::new("󰁍").size(18.0)).clicked() {
                        if self.current_month == 1 {
                            self.current_month = 12;
                            self.current_year -= 1;
                        } else {
                            self.current_month -= 1;
                        }
                    }

                    ui.label(
                        RichText::new(format!(
                            "{} {}",
                            month_name(self.current_month),
                            self.current_year
                        ))
                        .size(20.0),
                    );

                    if ui.button(RichText::new("󰁔").size(18.0)).clicked() {
                        if self.current_month == 12 {
                            self.current_month = 1;
                            self.current_year += 1;
                        } else {
                            self.current_month += 1;
                        }
                    }
                });

                ui.add_space(12.0);

                let cell_size = 46.0;
                let cell = Vec2::splat(cell_size);

                ui.horizontal(|ui| {
                    for day in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
                        ui.allocate_ui(Vec2::new(cell_size, 22.0), |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(RichText::new(day).strong());
                            });
                        });
                    }
                });

                let first_day =
                    NaiveDate::from_ymd_opt(self.current_year, self.current_month, 1).unwrap();
                let days_in_month = days_in_month(self.current_year, self.current_month);
                let start_weekday = first_day.weekday().num_days_from_monday() as usize;

                let mut day = 1;
                for week in 0..6 {
                    if day > days_in_month {
                        break;
                    }

                    ui.horizontal(|ui| {
                        for weekday in 0..7 {
                            if (week == 0 && weekday < start_weekday) || day > days_in_month {
                                ui.allocate_ui(cell, |_| {});
                            } else {
                                let date = NaiveDate::from_ymd_opt(
                                    self.current_year,
                                    self.current_month,
                                    day,
                                )
                                .unwrap();

                                let is_worked = self.data.worked_days.contains(&date);
                                let is_today = date == Local::now().date_naive();

                                let (bg_color, text_color) = if self.use_custom_theme {
                                    if is_worked {
                                        (
                                            Color32::from_rgb(196, 232, 129),
                                            Color32::from_rgb(20, 18, 22),
                                        )
                                    } else if is_today {
                                        (
                                            Color32::from_rgb(172, 130, 233),
                                            Color32::from_rgb(20, 18, 22),
                                        )
                                    } else {
                                        (Color32::TRANSPARENT, ui.style().visuals.text_color())
                                    }
                                } else {
                                    if is_worked {
                                        (Color32::from_rgb(40, 160, 70), Color32::WHITE)
                                    } else if is_today {
                                        (Color32::from_rgb(70, 110, 180), Color32::WHITE)
                                    } else {
                                        (Color32::TRANSPARENT, ui.style().visuals.text_color())
                                    }
                                };

                                let response = ui.allocate_response(cell, Sense::click());

                                if response.clicked() {
                                    self.toggle_day(date);
                                }

                                let rect = response.rect;
                                ui.painter().rect_filled(rect, 4.0, bg_color);

                                if response.hovered() {
                                    let hover_color = if self.use_custom_theme {
                                        Color32::from_rgb(172, 130, 233)
                                    } else {
                                        Color32::LIGHT_GRAY
                                    };

                                    ui.painter().rect_stroke(
                                        rect,
                                        4.0,
                                        egui::Stroke::new(1.5_f32, hover_color),
                                    );
                                }

                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    day.to_string(),
                                    egui::FontId::proportional(17.0),
                                    text_color,
                                );

                                day += 1;
                            }
                        }
                    });
                }

                ui.add_space(14.0);
                ui.label(format!("Marked days: {}", self.data.worked_days.len()));
                ui.label("Click a day to toggle mark");
            });
        });
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(
        if month == 12 { year + 1 } else { year },
        if month == 12 { 1 } else { month + 1 },
        1,
    )
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 440.0])
            .with_resizable(false)
            .with_decorations(false)
            .with_title(""),
        ..Default::default()
    };

    eframe::run_native(
        "Lainlog",
        options,
        Box::new(|cc| Ok(Box::new(WorkCalendar::new(cc)))),
    )
}

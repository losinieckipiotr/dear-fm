use egui::{
    CentralPanel, Color32, InnerResponse, Panel, RichText, ScrollArea, Ui,
    WidgetText,
};

use egui_extras::{Column, TableBuilder};

use crate::state::{AppState, Side};

const NUM_MANUAL_ROWS: usize = 20;

pub fn render_table(ui: &mut Ui, side: Side, state: &mut AppState) {
    let striped = true;
    let resizable = true;
    let clickable = true;
    let reset = false;
    let overline = false;

    // let text_height = egui::TextStyle::Body
    //     .resolve(ui.style())
    //     .size
    //     .max(ui.spacing().interact_size.y);

    let available_height = ui.available_height();

    let mut table = TableBuilder::new(ui)
        .striped(striped)
        .resizable(resizable)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto())
        .column(
            Column::remainder()
                .at_least(40.0)
                .clip(true)
                .resizable(true),
        )
        .column(Column::auto())
        .column(Column::remainder())
        .column(Column::remainder())
        .min_scrolled_height(0.0)
        .max_scroll_height(available_height);

    if clickable {
        table = table.sense(egui::Sense::click());
    }

    if let Some(row_index) = state.scroll_to_row.take() {
        table = table.scroll_to_row(row_index, None);
    }

    if reset {
        table.reset();
    }

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                egui::Sides::new().show(
                    ui,
                    |ui| {
                        ui.strong("Row");
                    },
                    |ui| {
                        state.reversed ^= ui
                            .button(if state.reversed { "⬆" } else { "⬇" })
                            .clicked();
                    },
                );
            });
            header.col(|ui| {
                ui.strong("Clipped text");
            });
            header.col(|ui| {
                ui.strong("Expanding content");
            });
            header.col(|ui| {
                ui.strong("Interaction");
            });
            header.col(|ui| {
                ui.strong("Content");
            });
        })
        .body(|mut body| {
            for row_index in 0..NUM_MANUAL_ROWS {
                let row_index = if state.reversed {
                    NUM_MANUAL_ROWS - 1 - row_index
                } else {
                    row_index
                };

                let is_thick = thick_row(row_index);
                let row_height = if is_thick { 30.0 } else { 18.0 };
                body.row(row_height, |mut row| {
                    row.set_selected(state.selection.contains(&row_index));
                    row.set_overline(overline && row_index % 7 == 3);

                    row.col(|ui| {
                        text_in_table(ui, row_index.to_string());
                    });
                    row.col(|ui| {
                        text_in_table(ui, long_text(row_index));
                    });
                    row.col(|ui| {
                        expanding_content(ui);
                    });
                    row.col(|ui| {
                        ui.checkbox(&mut state.checked, "Click me");
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode =
                            Some(egui::TextWrapMode::Extend);
                        if is_thick {
                            ui.heading("Extra thick row");
                        } else {
                            ui.label("Normal row");
                        }
                    });

                    toggle_row_selection(state, row_index, &row.response());
                });
            }
        });
}

fn toggle_row_selection(
    state: &mut AppState,
    row_index: usize,
    row_response: &egui::Response,
) {
    if row_response.clicked() {
        if state.selection.contains(&row_index) {
            state.selection.remove(&row_index);
        } else {
            state.selection.insert(row_index);
        }
    }
}

fn expanding_content(ui: &mut egui::Ui) {
    ui.add(egui::Separator::default().horizontal());
}

fn long_text(row_index: usize) -> String {
    format!(
        "Row {row_index} has some long text that you may want to clip, or it will take up too much horizontal space!"
    )
}

fn thick_row(row_index: usize) -> bool {
    row_index.is_multiple_of(6)
}
fn text_in_table(
    ui: &mut egui::Ui,
    text: impl Into<WidgetText>,
) -> InnerResponse<()> {
    ui.add_enabled_ui(false, |ui| {
        ui.set_opacity(1.0);
        ui.label(text);
    })
}

// DemoType::ManyHomogeneous => {
//     body.rows(text_height, self.num_rows, |mut row| {
//         let row_index = if self.reversed {
//             self.num_rows - 1 - row.index()
//         } else {
//             row.index()
//         };

//         row.set_selected(self.selection.contains(&row_index));
//         row.set_overline(self.overline && row_index % 7 == 3);

//         row.col(|ui| {
//             ui.label(row_index.to_string());
//         });
//         row.col(|ui| {
//             ui.label(long_text(row_index));
//         });
//         row.col(|ui| {
//             expanding_content(ui);
//         });
//         row.col(|ui| {
//             ui.checkbox(&mut self.checked, "Click me");
//         });
//         row.col(|ui| {
//             ui.add(
//                 egui::Label::new(
//                     "Thousands of rows of even height",
//                 )
//                 .wrap_mode(TextWrapMode::Extend),
//             );
//         });

//         self.toggle_row_selection(row_index, &row.response());
//     });
//     }
//     DemoType::ManyHeterogenous => {
//     let row_height =
//         |i: usize| if thick_row(i) { 30.0 } else { 18.0 };
//     body.heterogeneous_rows(
//         (0..self.num_rows).map(row_height),
//         |mut row| {
//             let row_index = if self.reversed {
//                 self.num_rows - 1 - row.index()
//             } else {
//                 row.index()
//             };

//             row.set_selected(self.selection.contains(&row_index));
//             row.set_overline(self.overline && row_index % 7 == 3);

//             row.col(|ui| {
//                 ui.label(row_index.to_string());
//             });
//             row.col(|ui| {
//                 ui.label(long_text(row_index));
//             });
//             row.col(|ui| {
//                 expanding_content(ui);
//             });
//             row.col(|ui| {
//                 ui.checkbox(&mut self.checked, "Click me");
//             });
//             row.col(|ui| {
//                 ui.style_mut().wrap_mode =
//                     Some(egui::TextWrapMode::Extend);
//                 if thick_row(row_index) {
//                     ui.heading("Extra thick row");
//                 } else {
//                     ui.label("Normal row");
//                 }
//             });

//             self.toggle_row_selection(row_index, &row.response());
//         },
//     );
//     }

use crate::view_models::MessageTableViewModel;
use eframe::egui::{CentralPanel, Color32, Label, Sense, TextEdit, TopBottomPanel, Ui, RichText};
use egui_extras::{Column, TableBuilder};

pub(crate) struct MessageTableView<'a> {
    model: &'a mut MessageTableViewModel,
}

impl<'a> MessageTableView<'a> {
    pub(crate) fn new(model: &'a mut MessageTableViewModel) -> Self {
        Self { model }
    }

    pub(crate) fn view(&mut self, ui: &mut Ui) {
        TopBottomPanel::bottom("message_filter_panel").show_inside(ui, |ui| {
            self.show_message_type_labels(ui);
            self.show_hash_input(ui);
            self.show_account_input(ui);
        });

        CentralPanel::default().show_inside(ui, |ui| {
            //ui.add_space(5.0);
            ui.heading(self.model.heading());
            self.show_message_table(ui);
        });
    }

    fn show_message_type_labels(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            let mut changed = false;
            for type_filter in &mut self.model.message_types {
                if ui
                    .selectable_label(type_filter.selected, type_filter.label.clone())
                    .clicked()
                {
                    type_filter.selected = !type_filter.selected;
                    changed = true;
                }
            }
            if changed {
                self.model.update_type_filter();
            }
        });
    }

    fn show_hash_input(&mut self, ui: &mut Ui) {
        let text_color = if self.model.hash_error {
            Some(Color32::RED)
        } else {
            None
        };
        if ui
            .add(
                TextEdit::singleline(&mut self.model.hash_filter)
                    .hint_text("block hash...")
                    .text_color_opt(text_color),
            )
            .changed()
        {
            self.model.update_hash_filter()
        };
    }

    fn show_account_input(&mut self, ui: &mut Ui) {
        let text_color = if self.model.account_error {
            Some(Color32::RED)
        } else {
            None
        };
        if ui
            .add(
                TextEdit::singleline(&mut self.model.account_filter)
                    .hint_text("account...")
                    .text_color_opt(text_color),
            )
            .changed()
        {
            self.model.update_account_filter()
        };
    }

    fn show_message_table(&mut self, ui: &mut Ui) {
        TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .auto_shrink(false)
            .sense(Sense::click())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Channel");
                });
                header.col(|ui| {
                    ui.strong("in/out");
                });
                header.col(|ui| {
                    ui.strong("Type");
                });
            })
            .body(|body| {
                body.rows(20.0, self.model.message_count(), |mut row| {
                    let Some(row_model) = self.model.get_row(row.index()) else {
                        return;
                    };
                    if row_model.is_selected {
                        row.set_selected(true);
                    }
                    row.col(|ui| {
                        ui.add(Label::new(row_model.channel_id).selectable(false));
                    });
                    row.col(|ui| {
                        ui.add(Label::new(row_model.direction).selectable(false));
                    });
                    row.col(|ui| {
                        ui.painter().rect_filled(
                            ui.available_rect_before_wrap(),
                            0.0,
                            row_model.background_color
                        );
                        ui.add(
                            Label::new(RichText::new(&row_model.message).color(row_model.text_color)).selectable(false)
                        );
                    });
                    if row.response().clicked() {
                        self.model.select_message(row.index());
                    }
                })
            });
    }
}

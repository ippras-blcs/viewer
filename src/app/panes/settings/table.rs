use std::hash::{Hash, Hasher};
use crate::app::MAX_PRECISION;
use egui::{ahash::{HashSet, HashSetExt}, ComboBox, Grid, PopupCloseBehavior, RichText, Slider, Ui};
use egui_ext::LabeledSeparator as _;
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::{FUNNEL, FUNNEL_X};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use polars_utils::{format_list_container_truncated, format_list_truncated};

/// Configuration settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) resizable: bool,
    pub(crate) editable: bool,
    pub(crate) precision: usize,
    pub(crate) sticky_columns: usize,
    pub(crate) truncate: bool,

    pub(crate) filter: Filter,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            resizable: false,
            editable: false,
            precision: 2,
            sticky_columns: 0,
            truncate: false,

            filter: Filter::new(),
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        Grid::new("Configuration").show(ui, |ui| {
            // Precision
            ui.label(ui.localize("precision"));
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            // // Sticky
            // ui.label(ui.localize("sticky_columns"));
            // ui.add(Slider::new(&mut self.sticky_columns));
            // ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Filter
            ui.separator();
            ui.labeled_separator(RichText::new(ui.localize("filter")).heading());
            ui.end_row();

            self.filter.show(ui, data_frame)?;
            ui.end_row();

            // Filter
            let mut response = ui.label(ui.localize("filter"));
            // response |= ui.checkbox(&mut self.names, "");
            ComboBox::new(id_salt, label)
            response.on_hover_localized("filter.hover");
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Filter {
    pub(crate) identifiers: HashSet<u64>,
}

impl Filter {
    pub(crate) fn show(&mut self, ui: &mut Ui, data_frame: &DataFrame) -> PolarsResult<()> {
        // Onset temperature filter
        ui.label(ui.localize("filter-by-identifier"))
            .on_hover_localized("filter-by-identifier.hover");
        // let text = format_list_truncated!(&self.onset_temperatures, 2);
        ComboBox::from_id_salt("Identifiers")
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .selected_text("text")
            .show_ui(ui, |ui| -> PolarsResult<()> {
                let identifiers = data_frame["Identifier"].u64()?.unique()?;
                for identifier in identifiers.iter().flatten() {
                    let checked = self.identifiers.contains(&identifier);
                    let response =
                        ui.selectable_label(checked, AnyValue::from(identifier).to_string());
                    if response.clicked() {
                        if checked {
                            self.identifiers.remove(&identifier);
                        } else {
                            self.identifiers.insert(identifier);
                        }
                    }
                    response.context_menu(|ui| {
                        if ui.button(format!("{FUNNEL} Select all")).clicked() {
                            self.identifiers = identifiers.iter().flatten().collect();
                            ui.close_menu();
                        }
                        if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
                            self.identifiers = HashSet::new();
                            ui.close_menu();
                        }
                    });
                }
                Ok(())
            })
            .inner
            .transpose()?;
        ui.end_row();

        // Temperature step filter
        ui.label(ui.localize("filter-by-temperature-step"))
            .on_hover_localized("filter-by-temperature-step.hover");
        let text = format_list_truncated!(&self.temperature_steps, 2);
        ComboBox::from_id_salt("TemperatureStepFilter")
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .selected_text(text)
            .show_ui(ui, |ui| -> PolarsResult<()> {
                let temperature_steps = data_frame.mode().temperature_step()?.unique();
                for temperature_step in temperature_steps.iter().flatten() {
                    let checked = self.temperature_steps.contains(&temperature_step);
                    let response =
                        ui.selectable_label(checked, AnyValue::from(temperature_step).to_string());
                    if response.clicked() {
                        if checked {
                            self.temperature_steps.remove_by_value(&temperature_step);
                        } else {
                            self.temperature_steps.push(temperature_step);
                        }
                    }
                    response.context_menu(|ui| {
                        if ui.button(format!("{FUNNEL} Select all")).clicked() {
                            self.temperature_steps = temperature_steps.iter().flatten().collect();
                            ui.close_menu();
                        }
                        if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
                            self.temperature_steps = Vec::new();
                            ui.close_menu();
                        }
                    });
                }
                Ok(())
            })
            .inner
            .transpose()?;
        ui.end_row();

        // Fatty acids filter
        ui.label(ui.localize("filter-by-fatty-acids"))
            .on_hover_localized("filter-by-fatty-acids.hover");
        let text = format_list_truncated!(
            self.fatty_acids
                .iter()
                .map(|fatty_acid| fatty_acid.display(COMMON)),
            2
        );
        let inner_response = ComboBox::from_id_salt("FattyAcidsFilter")
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .selected_text(text)
            .show_ui(ui, |ui| -> PolarsResult<()> {
                let fatty_acids = data_frame["FattyAcid"]
                    .unique()?
                    .sort(Default::default())?
                    .fa();
                for index in 0..fatty_acids.len() {
                    let Some(fatty_acid) = fatty_acids.get(index)? else {
                        continue;
                    };
                    let checked = self.identifiers.contains(&fatty_acid);
                    let response = ui
                        .selectable_label(checked, format!("{:#}", (&fatty_acid).display(COMMON)));
                    if response.clicked() {
                        if checked {
                            self.identifiers.remove_by_value(&fatty_acid);
                        } else {
                            self.identifiers.push(fatty_acid);
                        }
                    }
                    response.context_menu(|ui| {
                        if ui.button(format!("{FUNNEL} Select all")).clicked() {
                            self.identifiers = fatty_acids.clone().into_iter().flatten().collect();
                            ui.close_menu();
                        }
                        if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
                            self.identifiers = Vec::new();
                            ui.close_menu();
                        }
                    });
                }
                Ok(())
            });
        inner_response.inner.transpose()?;
        inner_response.response.on_hover_ui(|ui| {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
            ui.label(self.identifiers.len().to_string());
        });
        Ok(())
    }
}

impl Hash for Filter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.identifiers.len());

        let build_hasher = S::default();

        let hash = self.identifiers
            .iter()
            .map(|t| build_hasher.hash_one(t))
            .fold(0, u64::wrapping_add);

        state.write_u64(hash);
    }
}
use egui::{Color32, ComboBox, DragValue, RichText, Ui, Vec2b, emath::Float};
use egui_l20n::UiExt;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, hash::Hash};
use time::{UtcOffset, macros::offset};

/// Settings
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) drag: Vec2b,
    pub(crate) legend: bool,
    pub(crate) link: Vec2b,
    pub(crate) scroll: bool,
    pub(crate) zoom: Vec2b,
    // pub(crate) temperature: Temperature,
    // pub(crate) concentration: Concentration,
    pub(crate) time: Time,

    pub(crate) source: Source,
    pub(crate) resampling: Resampling,
    pub(crate) rolling: Rolling,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            drag: Vec2b { x: false, y: true },
            legend: true,
            link: Vec2b { x: false, y: false },
            scroll: false,
            zoom: Vec2b { x: false, y: true },
            time: Time {
                offset: UtcOffset::LOCAL,
            },
            // temperature: Temperature {
            //     unit: TemperatureUnit::DegreeCelsius,
            // },
            // concentration: Concentration {
            //     unit: ConcentrationUnit::MilligramPerCubicMeter,
            // },
            source: Source::new(),
            resampling: Resampling::new(),
            rolling: Rolling::new(),
        }
    }
}

// temperature: Temperature {
//     unit: TemperatureUnit::DegreeCelsius,
// },
// concentration: Concentration {
//     unit: ConcentrationUnit::MilligramPerCubicMeter,
// },
impl Settings {
    pub(crate) fn ui(&mut self, ui: &mut Ui) {
        ui.collapsing(RichText::new(ui.localize("plot")).heading(), |ui| {
            // ui.horizontal(|ui| {
            //     ui.label("Temperature unit:");
            //     ComboBox::from_id_source("temperature_unit")
            //         .selected_text(context.settings.temperature.unit.to_string())
            //         .show_ui(ui, |ui| {
            //             for unit in TEMPERATURE_UNITS {
            //                 ui.selectable_value(
            //                     &mut context.settings.temperature.unit,
            //                     unit,
            //                     unit.abbreviation(),
            //                 )
            //                 .on_hover_text(unit.singular());
            //             }
            //         })
            //         .response
            //         .on_hover_text(context.settings.temperature.unit.singular());
            // });
            // ui.horizontal(|ui| {
            //     ui.label("Concentration unit:");
            //     ComboBox::from_id_source("concentration_unit")
            //         .selected_text(context.settings.concentration.unit.abbreviation())
            //         .show_ui(ui, |ui| {
            //             for unit in CONCENTRATION_UNITS {
            //                 ui.selectable_value(
            //                     &mut context.settings.concentration.unit,
            //                     unit,
            //                     unit.abbreviation(),
            //                 )
            //                 .on_hover_text(unit.singular());
            //             }
            //         })
            //         .response
            //         .on_hover_text(context.settings.concentration.unit.singular());
            // });
            ui.horizontal(|ui| {
                ui.label(ui.localize("time_zone"));
                let mut changed = false;
                ComboBox::from_id_salt("time_zone")
                    .selected_text(self.time.offset.name())
                    .show_ui(ui, |ui| {
                        changed |= ui
                            .selectable_value(
                                &mut self.time.offset,
                                UtcOffset::UTC,
                                UtcOffset::UTC.name(),
                            )
                            .on_hover_text(ui.localize("time_zone__utc.hover"))
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut self.time.offset,
                                UtcOffset::LOCAL,
                                UtcOffset::LOCAL.name(),
                            )
                            .on_hover_text(ui.localize("time_zone__local.hover"))
                            .changed();
                    });
                // if changed && behavior.settings.link {
                //     self.settings.values_mut().for_each(
                //         |Settings { time, .. }| time.offset = settings.time.offset,
                //     );
                // }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(ui.localize("legend"));
                if ui
                    .checkbox(&mut self.legend, "")
                    .on_hover_text(ui.localize("legend.hover"))
                    .changed()
                {
                    // self.settings
                    //     .values_mut()
                    //     .for_each(|Settings { legend, .. }| *legend = settings.legend);
                }
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("link_axis"));
                if ui
                    .checkbox(&mut self.link.x, "")
                    .on_hover_text(ui.localize("link_axis.hover?axis=x"))
                    .changed()
                {
                    // self.settings
                    //     .values_mut()
                    //     .for_each(|Settings { link, .. }| link.x = settings.link.x);
                }
                if ui
                    .checkbox(&mut self.link.y, "")
                    .on_hover_text(ui.localize("link_axis.hover?axis=y"))
                    .changed()
                {
                    // self.settings
                    //     .values_mut()
                    //     .for_each(|Settings { link, .. }| link.y = settings.link.y);
                }
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("drag"));
                if ui
                    .checkbox(&mut self.drag.x, "")
                    .on_hover_text(ui.localize("drag.hover?axis=x"))
                    .changed()
                {
                    // self.settings
                    //     .values_mut()
                    //     .for_each(|Settings { drag, .. }| drag.x = settings.drag.x);
                }
                if ui
                    .checkbox(&mut self.drag.y, "")
                    .on_hover_text(ui.localize("drag.hover?axis=y"))
                    .changed()
                {
                    // self.settings
                    //     .values_mut()
                    //     .for_each(|Settings { drag, .. }| drag.y = settings.drag.y);
                }
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("scroll"));
                if ui
                    .checkbox(&mut self.scroll, "")
                    .on_hover_text(ui.localize("scroll.hover"))
                    .changed()
                {
                    // self.settings
                    //     .values_mut()
                    //     .for_each(|Settings { scroll, .. }| *scroll = settings.scroll);
                }
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("zoom"));
                if ui
                    .checkbox(&mut self.zoom.x, "")
                    .on_hover_text(ui.localize("zoom.hover?axis=x"))
                    .changed()
                {
                    // self.settings
                    //     .values_mut()
                    //     .for_each(|Settings { zoom, .. }| zoom.x = settings.zoom.x);
                }
                if ui
                    .checkbox(&mut self.zoom.y, "")
                    .on_hover_text(ui.localize("zoom.hover?axis=y"))
                    .changed()
                {
                    // self.settings
                    //     .values_mut()
                    //     .for_each(|Settings { zoom, .. }| zoom.y = settings.zoom.y);
                }
            });
        });
        ui.collapsing(RichText::new(ui.localize("source")).heading(), |ui| {
            ui.horizontal(|ui| {
                ui.label(ui.localize("points"));
                if ui
                    .add(
                        DragValue::new(&mut self.source.points.radius)
                            .range(0.0..=f32::MAX)
                            .speed(0.1),
                    )
                    .on_hover_text(ui.localize("points__radius.hover"))
                    .changed()
                {
                    // self.settings
                    //     .values_mut()
                    //     .for_each(|Settings { points, .. }| {
                    //         points.radius = settings.points.radius
                    //     });
                }
                if ui
                    .checkbox(&mut self.source.points.filled, "")
                    .on_hover_text(ui.localize("points__fill.hover"))
                    .changed()
                {
                    // self.settings
                    //     .values_mut()
                    //     .for_each(|Settings { points, .. }| {
                    //         points.filled = settings.points.filled
                    //     });
                }
                if ui
                    .color_edit_button_srgba(&mut self.source.points.color)
                    .on_hover_text(ui.localize("points__color.hover"))
                    .changed()
                {
                    // self.settings
                    //     .values_mut()
                    //     .for_each(|Settings { points, .. }| {
                    //         points.color = settings.points.color
                    //     });
                }
            });
        });
        ui.collapsing(RichText::new(ui.localize("resampling")).heading(), |ui| {
            ui.horizontal(|ui| {
                ui.label(ui.localize("resampling_mean"));
                ui.checkbox(&mut self.resampling.mean, "")
                    .on_hover_text(ui.localize("resampling_mean.hover"));
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("resampling_median"));
                ui.checkbox(&mut self.resampling.median, "")
                    .on_hover_text(ui.localize("resampling_median.hover"));
            });
            if !self.resampling.mean && !self.resampling.median {
                ui.disable();
            }
            ui.horizontal(|ui| {
                ui.label(ui.localize("every"));
                ui.add(DragValue::new(&mut self.resampling.every).range(1..=86400))
                    .on_hover_text(ui.localize("every.hover"));
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("period"));
                ui.add(DragValue::new(&mut self.resampling.period).range(1..=86400))
                    .on_hover_text(ui.localize("window_duration"));
            });
        });
        ui.collapsing(RichText::new(ui.localize("rolling")).heading(), |ui| {
            ui.horizontal(|ui| {
                ui.label(ui.localize("mean"));
                ui.checkbox(&mut self.rolling.mean, "")
                    .on_hover_text(ui.localize("rolling_mean"));
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("median"));
                ui.checkbox(&mut self.rolling.median, "")
                    .on_hover_text(ui.localize("rolling_median"));
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("window_size"));
                ui.add(DragValue::new(&mut self.rolling.window_size).range(1..=usize::MAX))
                    .on_hover_text(ui.localize("window_size_description"));
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("min_periods"));
                ui.add(
                    DragValue::new(&mut self.rolling.min_periods)
                        .range(1..=self.rolling.window_size),
                )
                .on_hover_text(ui.localize("min_periods_description"));
            });
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for Settings {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.drag.x.hash(state);
        self.drag.y.hash(state);
        self.legend.hash(state);
        self.link.x.hash(state);
        self.link.y.hash(state);
        self.scroll.hash(state);
        self.zoom.x.hash(state);
        self.zoom.y.hash(state);
        self.time.hash(state);
        self.source.hash(state);
        self.resampling.hash(state);
        self.rolling.hash(state);
    }
}

/// Source
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Source {
    pub(crate) points: Points,
}

impl Source {
    pub(crate) const fn new() -> Self {
        Self {
            points: Points {
                color: Color32::TRANSPARENT,
                filled: true,
                radius: 0.0,
            },
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Self::new()
    }
}

/// Downsampling
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Resampling {
    pub(crate) mean: bool,
    pub(crate) median: bool,
    pub(crate) every: i64,
    pub(crate) period: i64,
}

impl Resampling {
    pub(crate) const fn new() -> Self {
        Self {
            mean: true,
            median: false,
            every: 60,
            period: 120,
        }
    }
}

impl Default for Resampling {
    fn default() -> Self {
        Self::new()
    }
}

/// Rolling
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Rolling {
    pub(crate) mean: bool,
    pub(crate) median: bool,
    pub(crate) window_size: usize,
    pub(crate) min_periods: usize,
}

impl Rolling {
    pub(crate) const fn new() -> Self {
        Self {
            mean: false,
            median: false,
            window_size: 30,
            min_periods: 1,
        }
    }
}

impl Default for Rolling {
    fn default() -> Self {
        Self::new()
    }
}

// /// Concentration
// #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
// pub(crate) struct Concentration {
//     pub(crate) unit: ConcentrationUnit,
// }

/// Points
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Points {
    pub(crate) color: Color32,
    pub(crate) filled: bool,
    pub(crate) radius: f32,
}

impl Hash for Points {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.filled.hash(state);
        self.radius.ord().hash(state);
    }
}

// /// Temperature
// #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
// pub(crate) struct Temperature {
//     pub(crate) unit: TemperatureUnit,
// }

/// Time
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Time {
    pub(crate) offset: UtcOffset,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            offset: UtcOffset::UTC,
        }
    }
}

/// Offset
pub trait Offset {
    const LOCAL: UtcOffset = offset!(+3);

    fn name(&self) -> Cow<str>;
}

impl Offset for UtcOffset {
    fn name(&self) -> Cow<str> {
        if *self == UtcOffset::UTC {
            Cow::Borrowed("UTC")
        } else {
            Cow::Owned(self.to_string())
        }
    }
}

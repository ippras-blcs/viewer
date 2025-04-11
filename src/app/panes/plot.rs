use super::settings::plot::Settings;
use crate::app::computers::{PlotComputed, PlotKey};
use egui::{Id, Response, Ui, Widget, emath::round_to_decimals};
use egui_plot::{GridInput, GridMark, Legend, Line, Plot, PlotPoints, Points};
use polars::prelude::*;
use std::{
    iter::{self},
    ops::Range,
};
use time::{OffsetDateTime, UtcOffset, format_description::FormatItem, macros::format_description};

const SECOND: f64 = 1_f64;
const MINUTE: f64 = 60_f64 * SECOND;
const HOUR: f64 = 60_f64 * MINUTE;
const DAY: f64 = 24_f64 * HOUR;
const DAYS10: f64 = 10_f64 * DAY;
const DAYS100: f64 = 100_f64 * DAY;
const DAYS1000: f64 = 1000_f64 * DAY;

static HMS: &[FormatItem] = format_description!("[hour]:[minute]:[second]");
static YMD: &[FormatItem] = format_description!("[year]-[month]-[day]");
static YMDHMS: &[FormatItem] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]"
);

/// Plot pane
#[derive(Debug, PartialEq)]
pub(crate) struct Pane<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a mut Settings,
}

impl Widget for Pane<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        // Plot
        let mut plot = Plot::new("plot");
        if self.settings.legend {
            plot = plot.legend(Legend::default());
        }
        plot.label_formatter(|name, value| {
            let mut formatted = String::new();
            if !name.is_empty() {
                formatted.push_str(&format!("File: {name}\n"));
            }
            let time = format_time(value.x, self.settings.time.offset, YMDHMS);
            let temperature = round_to_decimals(value.y, 2);
            formatted.push_str(&format!("x = {time}\ny = {temperature}"));
            formatted
        })
        .allow_drag(self.settings.drag)
        .allow_scroll(self.settings.scroll)
        .allow_zoom(self.settings.zoom)
        // .x_axis_label("Time")
        .x_axis_formatter(|grid_mark, _| time_axis_formatter(grid_mark, self.settings.time.offset))
        .x_grid_spacer(|grid_input| time_grid_spacer(grid_input, self.settings.time.offset))
        // .y_axis_label(unit.abbreviation())
        // .y_axis_formatter(move |y, _| round_to_decimals(y.value, 2).to_string())
        .link_axis(Id::new("plot"), self.settings.link)
        .link_cursor(Id::new("plot"), self.settings.link)
        .show(ui, |ui| {
            let computed = ui.ctx().memory_mut(|memory| {
                memory.caches.cache::<PlotComputed>().get(PlotKey {
                    data_frame: self.data_frame,
                    settings: self.settings,
                })
            });
            // Source
            // Line
            ui.line(Line::new(PlotPoints::new(computed.source.clone())).name("Source line"));
            // Points
            if self.settings.source.points.radius > 0.0 {
                ui.points(
                    Points::new(computed.source)
                        .color(self.settings.source.points.color)
                        .filled(self.settings.source.points.filled)
                        .radius(self.settings.source.points.radius)
                        .name("Source points"),
                );
            }
            // Resampling
            if let Some(points) = computed.resampling.mean {
                ui.line(Line::new(PlotPoints::new(points)).name(format!("Resampling mean")));
            }
            if let Some(points) = computed.resampling.median {
                ui.line(Line::new(PlotPoints::new(points)).name(format!("Resampling median")));
            }
            // Rolling
            if let Some(points) = computed.rolling.mean {
                ui.line(Line::new(PlotPoints::new(points)).name(format!("Rolling mean")));
            }
            if let Some(points) = computed.rolling.median {
                ui.line(Line::new(PlotPoints::new(points)).name(format!("Rolling median")));
            }
            // // Resampling
            // if self.settings.resampling.enable {
            //     let data_frame = ui.ctx().memory_mut(|memory| {
            //         memory.caches.cache::<Resampled>().get(ResamplerKey {
            //             data_frame: &data_frame,
            //             resampling: &self.settings.resampling,
            //         })
            //     });
            //     let points = points(&data_frame, name)?;
            //     let line = Line::new(PlotPoints::new(points.clone())).name(format!("Resampling"));
            //     ui.line(line);
            // }
            // // Rolling
            // if self.settings.rolling.mean || self.settings.rolling.median {
            //     let data_frame = ui.ctx().memory_mut(|memory| {
            //         memory.caches.cache::<Rolled>().get(RollerKey {
            //             data_frame: &data_frame,
            //             rolling: &self.settings.rolling,
            //         })
            //     });
            //     // Mean
            //     if self.settings.rolling.mean {
            //         let points = points(&data_frame, &format!("{name}.RollingMean"))?;
            //         let line = Line::new(PlotPoints::new(points)).name("Rolling mean");
            //         ui.line(line);
            //         // .filter_map(|(milliseconds, value)| {
            //         //     Some([(milliseconds? as f64 / 1000f64) as _, value?])
            //         // })
            //     }
            //     // Median
            //     if self.settings.rolling.median {
            //         let points = points(&data_frame, &format!("{name}.RollingMedian"))?;
            //         let line = Line::new(PlotPoints::new(points)).name("Rolling median");
            //         ui.line(line);
            //     }
            // }
            Ok::<_, PolarsError>(())
        })
        .response
    }
}

impl Pane<'_> {
    pub(crate) fn settings(&mut self, ui: &mut Ui) {
        // self.settings.ui(ui)
    }
}

fn time_grid_spacer(grid_input: GridInput, offset: UtcOffset) -> Vec<GridMark> {
    let mut grid_marks = Vec::new();
    let (min, max) = grid_input.bounds;
    let duration = (max - min).round();
    let offset_seconds = offset.whole_seconds() as f64;
    let start = min.floor() - offset_seconds;
    let end = max.ceil() - offset_seconds;
    if duration < 2f64 * MINUTE {
        for value in time_grid_steps(start..end, SECOND) {
            grid_marks.push(GridMark {
                value: value + offset_seconds,
                step_size: SECOND,
            });
        }
    }
    if duration < 2f64 * HOUR {
        for value in time_grid_steps(start..end, MINUTE) {
            grid_marks.push(GridMark {
                value: value + offset_seconds,
                step_size: MINUTE,
            });
        }
    }
    if duration < 2f64 * DAY {
        for value in time_grid_steps(start..end, HOUR) {
            grid_marks.push(GridMark {
                value: value + offset_seconds,
                step_size: HOUR,
            });
        }
    }
    if duration < 2f64 * DAYS10 {
        for value in time_grid_steps(start..end, DAY) {
            grid_marks.push(GridMark {
                value: value - offset_seconds,
                step_size: DAY,
            });
        }
    }
    if duration < 2f64 * DAYS100 {
        for value in time_grid_steps(start..end, DAYS10) {
            grid_marks.push(GridMark {
                value: value - offset_seconds,
                step_size: DAYS10,
            });
        }
    }
    if duration < 2f64 * DAYS1000 {
        for value in time_grid_steps(start..end, DAYS100) {
            grid_marks.push(GridMark {
                value: value - offset_seconds,
                step_size: DAYS100,
            });
        }
    }
    if duration < 20_f64 * DAYS1000 {
        for value in time_grid_steps(start..end, DAYS1000) {
            grid_marks.push(GridMark {
                value: value - offset_seconds,
                step_size: DAYS1000,
            });
        }
    }
    grid_marks
}

fn time_grid_steps(range: Range<f64>, step: f64) -> impl Iterator<Item = f64> {
    let mut value = (range.start / step).ceil();
    let end = (range.end / step).floor();
    iter::from_fn(move || {
        if value <= end {
            Some((value, value += 1f64).0 * step)
        } else {
            None
        }
    })
}

fn time_axis_formatter(grid_mark: GridMark, offset: UtcOffset) -> String {
    let offset_date_time = offset_date_time(grid_mark.value, offset);
    match grid_mark.step_size {
        SECOND | MINUTE | HOUR => offset_date_time.format(HMS).unwrap_or_default(),
        DAY | DAYS10 | DAYS100 | DAYS1000 => offset_date_time.format(YMD).unwrap_or_default(),
        _ => String::new(),
    }
}

fn format_time(value: f64, offset: UtcOffset, format: &[FormatItem]) -> String {
    offset_date_time(value, offset)
        .format(format)
        .unwrap_or_default()
}

fn offset_date_time(value: f64, offset: UtcOffset) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(value as _)
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .to_offset(offset)
}

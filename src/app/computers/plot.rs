use crate::app::panes::settings::plot::Settings;
use egui::{
    emath::OrderedFloat,
    util::cache::{ComputerMut, FrameCache},
};
use polars::prelude::*;
use std::{
    hash::{Hash, Hasher},
    iter::zip,
};

const ROUND_DECIMALS: u32 = 6;

/// Plot computed
pub(in crate::app) type Computed = FrameCache<Value, Computer>;

/// Plot computer
#[derive(Default)]
pub(in crate::app) struct Computer;

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        let mut value = Value::default();
        let lazy_frame = key
            .data_frame
            .clone()
            .lazy()
            .sort(["Time"], Default::default());
        // Source
        let data_frame = lazy_frame
            .clone()
            .with_column(col("Time").cast(DataType::Float64) / lit(1000))
            .collect()
            .unwrap();
        let times = data_frame["Time"].f64().unwrap();
        let values = data_frame[1].f64().unwrap();
        value.source = zip(times, values)
            .filter_map(|(time, value)| Some([time?, value?]))
            .collect();
        // Resampling
        if key.settings.resampling.mean || key.settings.resampling.median {
            let every = Duration::parse(&format!("{}s", key.settings.resampling.every));
            let period = Duration::parse(&format!("{}s", key.settings.resampling.period));
            let mut aggs = vec![];
            if key.settings.resampling.mean {
                aggs.push(nth(1).mean().round(ROUND_DECIMALS).alias("ResamplingMean"));
            }
            if key.settings.resampling.median {
                aggs.push(
                    nth(1)
                        .median()
                        .round(ROUND_DECIMALS)
                        .alias("ResamplingMedian"),
                );
            }
            let data_frame = lazy_frame
                .clone()
                .group_by_dynamic(
                    col("Time"),
                    [],
                    DynamicGroupOptions {
                        every,
                        period,
                        offset: Duration::parse("0"),
                        ..Default::default()
                    },
                )
                .agg(aggs)
                .with_column(col("Time").cast(DataType::Float64) / lit(1000))
                .collect()
                .unwrap();
            let times = data_frame["Time"].f64().unwrap();
            if key.settings.resampling.mean {
                let values = data_frame["ResamplingMean"].f64().unwrap();
                value.resampling.mean = zip(times, values)
                    .map(|(time, value)| Some([time?, value?]))
                    .collect();
            }
            if key.settings.resampling.median {
                let values = data_frame["ResamplingMedian"].f64().unwrap();
                value.resampling.median = zip(times, values)
                    .map(|(time, value)| Some([time?, value?]))
                    .collect();
            }
        }
        // Rolling
        if key.settings.rolling.mean || key.settings.rolling.median {
            let mut exprs = vec![];
            if key.settings.rolling.mean {
                exprs.push(
                    nth(1)
                        .rolling_mean(RollingOptionsFixedWindow {
                            window_size: key.settings.rolling.window_size,
                            min_periods: key.settings.rolling.min_periods,
                            ..Default::default()
                        })
                        .round(ROUND_DECIMALS)
                        .alias("RollingMean"),
                );
            }
            if key.settings.rolling.median {
                exprs.push(
                    nth(1)
                        .rolling_median(RollingOptionsFixedWindow {
                            window_size: key.settings.rolling.window_size,
                            min_periods: key.settings.rolling.min_periods,
                            ..Default::default()
                        })
                        .round(ROUND_DECIMALS)
                        .alias("RollingMedian"),
                );
            }
            let data_frame = lazy_frame
                .with_columns(exprs)
                .with_column(col("Time").cast(DataType::Float64) / lit(1000))
                .collect()
                .unwrap();
            let times = data_frame["Time"].f64().unwrap();
            if key.settings.rolling.mean {
                let values = data_frame["RollingMean"].f64().unwrap();
                value.rolling.mean = zip(times, values)
                    .map(|(time, value)| Some([time?, value?]))
                    .collect();
            }
            if key.settings.rolling.median {
                let values = data_frame["RollingMedian"].f64().unwrap();
                value.rolling.median = zip(times, values)
                    .map(|(time, value)| Some([time?, value?]))
                    .collect();
            }
        }
        value
    }
}

/// Key
#[derive(Clone, Copy, Debug)]
pub(in crate::app) struct Key<'a> {
    pub(in crate::app) data_frame: &'a DataFrame,
    pub(in crate::app) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Ok(times) = self.data_frame[0].str() {
            for time in times {
                time.hash(state);
            }
        }
        if let Ok(values) = self.data_frame[1].f64() {
            for value in values {
                value.map(OrderedFloat).hash(state);
            }
        }
        self.settings.hash(state);
    }
}

/// Value
// type Value = DataFrame;
// type Value = Vec<[f64; 2]>;
#[derive(Clone, Debug, Default)]
pub(in crate::app) struct Value {
    pub(in crate::app) source: Vec<[f64; 2]>,
    pub(in crate::app) resampling: Resampling,
    pub(in crate::app) rolling: Rolling,
}

#[derive(Clone, Debug, Default)]
pub(in crate::app) struct Resampling {
    pub(in crate::app) mean: Option<Vec<[f64; 2]>>,
    pub(in crate::app) median: Option<Vec<[f64; 2]>>,
}

#[derive(Clone, Debug, Default)]
pub(in crate::app) struct Rolling {
    pub(in crate::app) mean: Option<Vec<[f64; 2]>>,
    pub(in crate::app) median: Option<Vec<[f64; 2]>>,
}

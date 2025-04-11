use super::{ID_SOURCE, settings::table::Settings, state::State};
use egui::{
    Context, Direction, Frame, Id, Layout, Margin, Response, Sense, TextStyle, TextWrapMode, Ui,
    Vec2, Widget, vec2,
};
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::{HASH, MINUS};
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use polars::prelude::*;
use tracing::instrument;

const MARGIN: Vec2 = vec2(4.0, 2.0);

const INDEX: usize = 0;
const IDENTIFIER: usize = 1;
const VALUE: usize = 2;
const TIMESTAMP: usize = 3;
const LEN: usize = 4;

/// Table pane
#[derive(Debug)]
pub(crate) struct TableView<'a> {
    data_frame: &'a DataFrame,
    settings: &'a Settings,
    state: &'a mut State,
}

// impl Widget for TableView<'_> {
//     fn ui(self, ui: &mut Ui) -> Response {
//         let width = ui.spacing().interact_size.x;
//         let height = ui.spacing().interact_size.y;
//         let data_frame = self.data_frame;
//         ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
//         TableBuilder::new(ui)
//             .column(Column::auto_with_initial_suggestion(width))
//             .columns(Column::auto(), 3)
//             .auto_shrink(false)
//             .striped(true)
//             .header(height, |mut row| {
//                 row.col(|ui| {
//                     ui.heading("Index");
//                 });
//                 row.col(|ui| {
//                     ui.heading("Identifier");
//                 });
//                 row.col(|ui| {
//                     let text = data_frame.get_column_names()[1].as_str();
//                     ui.heading(text);
//                 });
//                 row.col(|ui| {
//                     ui.heading("Time");
//                 });
//             })
//             .body(|body| {
//                 let time = data_frame.time();
//                 let value = data_frame.value();
//                 let identifier = data_frame["Identifier"].u64().unwrap();
//                 let total_rows = data_frame.height();
//                 body.rows(height, total_rows, |mut row| {
//                     let row_index = row.index();
//                     // Index
//                     row.col(|ui| {
//                         ui.label(row_index.to_string());
//                     });
//                     // Identifier
//                     row.col(|ui| {
//                         let text = format!("{:x}", identifier.get(row_index).unwrap());
//                         ui.label(text);
//                     });
//                     // Value
//                     row.col(|ui| {
//                         let text = value.get(row_index).unwrap().to_string();
//                         ui.label(text);
//                     });
//                     // Time
//                     row.col(|ui| {
//                         let text = time.get(row_index).unwrap();
//                         ui.label(text);
//                     });
//                 });
//             });
//         ui.allocate_response(Default::default(), Sense::hover())
//     }
// }

impl<'a> TableView<'a> {
    pub(crate) fn new(
        data_frame: &'a DataFrame,
        settings: &'a Settings,
        state: &'a mut State,
    ) -> Self {
        Self {
            data_frame,
            settings,
            state,
        }
    }
}

impl TableView<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.state.reset_table_state {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.state.reset_table_state = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data_frame.height() as u64;
        let num_columns = LEN;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default().resizable(self.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.sticky_columns)
            .headers([HeaderRow::new(height)])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: usize) {
        if self.settings.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            (0, INDEX) => {
                ui.heading(HASH).on_hover_localized("index.hover");
            }
            (0, IDENTIFIER) => {
                ui.heading(ui.localize("identifier"))
                    .on_hover_localized("identifier.hover");
            }
            (0, VALUE) => {
                ui.heading(ui.localize("value"))
                    .on_hover_localized("value.hover");
            }
            (0, TIMESTAMP) => {
                ui.heading(ui.localize("timestamp"))
                    .on_hover_localized("timestamp.hover");
            }
            _ => {}
        };
    }

    #[instrument(skip(ui), err)]
    fn cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: usize) -> PolarsResult<()> {
        match (row, column) {
            (row, INDEX) => {
                ui.label(row.to_string());
            }
            (row, IDENTIFIER) => {
                let identifier = self.data_frame["Identifier"].u64()?;
                if let Some(identifier) = identifier.get(row) {
                    ui.label(format!("{identifier:x}"));
                }
            }
            (row, VALUE) => {
                let temperature = self.data_frame[1].f32()?;
                if let Some(temperature) = temperature.get(row) {
                    ui.label(temperature.to_string());
                }
            }
            (row, TIMESTAMP) => {
                let timestamp = self.data_frame["Timestamp"]
                    .datetime()?
                    .to_string("%Y-%m-%d %H:%M:%S")?;
                if let Some(timestamp) = timestamp.get(row) {
                    ui.label(timestamp.to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl TableDelegate for TableView<'_> {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.col_range.start)
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr % 2 == 0 {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr).ok();
            });
    }

    fn row_top_offset(&self, ctx: &Context, _table_id: Id, row_nr: u64) -> f32 {
        row_nr as f32 * (ctx.style().spacing.interact_size.y + 2.0 * MARGIN.y)
    }
}

// /// Extension methods for [`DataFrame`]
// trait DataFrameExt {
//     fn time(&self) -> ChunkedArray<StringType>;

//     fn try_time(&self) -> PolarsResult<ChunkedArray<StringType>>;

//     fn try_value(&self) -> PolarsResult<&ChunkedArray<Float32Type>>;

//     fn value(&self) -> &ChunkedArray<Float32Type>;
// }

// impl DataFrameExt for DataFrame {
//     fn time(&self) -> ChunkedArray<StringType> {
//         self.try_time().unwrap()
//     }

//     fn try_time(&self) -> PolarsResult<ChunkedArray<StringType>> {
//         // self["Timestamp"].datetime()?.to_string("%Y-%m-%d")
//         self["Timestamp"].datetime()?.to_string("%Y-%m-%d %H:%M:%S")
//     }

//     fn try_value(&self) -> PolarsResult<&ChunkedArray<Float32Type>> {
//         self[1].f32()
//     }

//     fn value(&self) -> &ChunkedArray<Float32Type> {
//         self.try_value().unwrap()
//     }
// }

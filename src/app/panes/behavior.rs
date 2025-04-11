use super::Pane;
use crate::utils::ContainerExt;
use egui::{
    CentralPanel, CollapsingHeader, CursorIcon, RichText, ScrollArea, Sides, TextStyle,
    TopBottomPanel, Ui, Vec2, WidgetText, menu::bar, vec2,
};
use egui_l20n::UiExt;
use egui_phosphor::regular::{LINK, X};
use egui_tiles::{Tile, TileId, Tiles, Tree, UiResponse};
use serde::{Deserialize, Serialize};

const SIZE: f32 = 16.0;
const MARGIN: Vec2 = vec2(4.0, 2.0);

/// Behavior
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Behavior {
    pub(crate) close: Option<TileId>,
}

impl Behavior {
    pub(crate) fn new() -> Self {
        Self { close: None }
    }
}

// impl Behavior {
//     pub(crate) fn settings(&mut self, ui: &mut Ui, tree: &mut Tree<Pane>) {
//         bar(ui, |ui| {
//             ui.toggle_value(&mut self.settings.link, RichText::new(LINK).size(SIZE))
//                 .on_hover_text(ui.localize("link_panes_settings"));
//         });
//         ui.separator();
//         for tile_id in tree.active_tiles() {
//             if let Some(Tile::Pane(pane)) = tree.tiles.get_mut(tile_id) {
//                 ui.visuals_mut().collapsing_header_frame = true;
//                 let open = self
//                     .click
//                     .take_if(|toggle| *toggle == tile_id)
//                     .map(|tile_id| {
//                         let id = ui.make_persistent_id(tile_id);
//                         ui.data_mut(|data| {
//                             let open = data.get_persisted_mut_or_default::<bool>(id);
//                             *open = !*open;
//                             *open
//                         })
//                     });
//                 CollapsingHeader::new(RichText::new(pane.title()).heading())
//                     .id_salt(tile_id)
//                     .open(open)
//                     .show(ui, |ui| {
//                         pane.settings(ui);
//                     });
//             }
//         }
//     }
// }

impl egui_tiles::Behavior<Pane> for Behavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> WidgetText {
        pane.name().into()
    }

    fn tab_title_for_tile(&mut self, tiles: &Tiles<Pane>, tile_id: TileId) -> WidgetText {
        if let Some(tile) = tiles.get(tile_id) {
            match tile {
                Tile::Pane(pane) => self.tab_title_for_pane(pane),
                Tile::Container(container) => {
                    if let Some(pane) = container.find_child_pane(tiles) {
                        format!("{}, ...", self.tab_title_for_pane(pane).text()).into()
                    } else {
                        format!("{:?}", container.kind()).into()
                    }
                }
            }
        } else {
            "MISSING TILE".into()
        }
    }

    fn pane_ui(&mut self, ui: &mut Ui, tile_id: TileId, pane: &mut Pane) -> UiResponse {
        let response = TopBottomPanel::top(ui.auto_id_with("TopPanel"))
            .show_inside(ui, |ui| {
                bar(ui, |ui| {
                    ScrollArea::horizontal()
                        .show(ui, |ui| {
                            ui.visuals_mut().button_frame = false;
                            Sides::new()
                                .height(ui.text_style_height(&TextStyle::Heading) + 4.0 * MARGIN.y)
                                .show(
                                    ui,
                                    |ui| pane.header(ui),
                                    |ui| {
                                        ui.visuals_mut().button_frame = false;
                                        if ui.button(RichText::new(X).heading()).clicked() {
                                            self.close = Some(tile_id);
                                        }
                                    },
                                )
                                .0
                        })
                        .inner
                })
                .inner
            })
            .inner;
        CentralPanel::default().show_inside(ui, |ui| {
            pane.body(ui);
        });
        if response.dragged() {
            UiResponse::DragStarted
        } else {
            UiResponse::None
        }
        // let response = ui
        //     .horizontal(|ui| {
        //         let response = ui.heading(pane.title()).on_hover_cursor(CursorIcon::Grab);
        //         ui.add_space(ui.available_width() - ui.spacing().button_padding.x - SIZE);
        //         ui.visuals_mut().button_frame = false;
        //         if ui.button(RichText::new(X)).clicked() {
        //             self.close = Some(tile_id);
        //         }
        //         response
        //     })
        //     .inner;
        // if response.clicked() {
        //     self.click = Some(tile_id);
        // }
        // pane.ui(ui);
        // if response.dragged() {
        //     UiResponse::DragStarted
        // } else {
        //     UiResponse::None
        // }
    }
}

/// Behavior settings
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) link: bool,
}

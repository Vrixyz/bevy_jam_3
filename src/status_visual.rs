use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_mod_picking::{Highlighting, Selection};

use crate::{
    progress::{self, Progress},
    InheritedBlockStatus, MapAssets, SelfBlockStatus,
};

pub fn update_status_visual(
    map_assets: Res<MapAssets>,
    mut q_status: Query<(
        &SelfBlockStatus,
        &Progress,
        &InheritedBlockStatus,
        &mut Highlighting<ColorMaterial>,
        &mut Selection,
    )>,
) {
    for (self_status, p, inherited_status, mut highlighting, mut selection) in q_status.iter_mut() {
        if p.timer.finished() == false || inherited_status.is_blocked {
            if highlighting.pressed != map_assets.node_materials_blocked.pressed {
                highlighting.apply(&map_assets.node_materials_blocked);
                selection.as_mut();
            }
        } else {
            if highlighting.pressed != map_assets.node_materials_normal.pressed {
                highlighting.apply(&map_assets.node_materials_normal);
                selection.as_mut();
            }
        }
    }
}

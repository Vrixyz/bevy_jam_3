use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_mod_picking::{Highlighting, Selection};

use crate::{
    new_node::EyeCatcher,
    progress::{self, Progress},
    InheritedBlockStatus, MapAssets, NodeManualBlockToggle, SelfBlockStatus,
};

pub fn update_status_visual(
    map_assets: Res<MapAssets>,
    mut q_status: Query<(
        &SelfBlockStatus,
        &Progress,
        &InheritedBlockStatus,
        Option<&NodeManualBlockToggle>,
        &mut Highlighting<ColorMaterial>,
        &mut Selection,
        &EyeCatcher,
    )>,
    mut q_visibility: Query<&mut Visibility>,
) {
    for (self_status,
        p,
        inherited_status,
        manual,
        mut highlighting,
        mut selection,
        eye_catcher)
        // long
        in
        q_status.iter_mut()
    {
        if !p.timer.finished() || inherited_status.is_blocked {
            if highlighting.pressed != map_assets.node_materials_blocked.pressed {
                *q_visibility.get_mut(eye_catcher.0).unwrap() = Visibility::Hidden;
                highlighting.apply(&map_assets.node_materials_blocked);
                selection.as_mut();
            }

            if manual.is_some() && self_status.is_blocked {
                *q_visibility.get_mut(eye_catcher.0).unwrap() = Visibility::Hidden;
            }
        } else {
            *q_visibility.get_mut(eye_catcher.0).unwrap() = Visibility::Visible;
            if highlighting.pressed != map_assets.node_materials_normal.pressed {
                highlighting.apply(&map_assets.node_materials_normal);
                selection.as_mut();
            }
            if manual.is_some() && !self_status.is_blocked {
                *q_visibility.get_mut(eye_catcher.0).unwrap() = Visibility::Hidden;
            }
            else {
                *q_visibility.get_mut(eye_catcher.0).unwrap() = Visibility::Visible;
            }
        }
    }
}

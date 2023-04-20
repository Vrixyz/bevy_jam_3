use bevy::{prelude::*};
use bevy_picking_highlight::*;

use crate::{
    new_node::EyeCatcher,
    picking::HighlightingMaterials,
    progress::{Progress},
    InheritedBlockStatus, NodeManualBlockToggle, SelfBlockStatus,
};

pub fn update_status_visual(
    highlighting_mats: Res<HighlightingMaterials>,
    mut q_status: Query<(
        &SelfBlockStatus,
        &Progress,
        &InheritedBlockStatus,
        Option<&NodeManualBlockToggle>,
        &mut HighlightOverride<ColorMaterial>,
        &EyeCatcher,
    )>,
    mut q_visibility: Query<&mut Visibility>,
) {
    for (self_status,
        p,
        inherited_status,
        manual,
        mut highlighting,
        eye_catcher)
        // long
        in
        q_status.iter_mut()
    {
        if !p.timer.finished() || inherited_status.is_blocked {
            let Some(HighlightKind::<ColorMaterial>::Fixed(current_highlight)) = &highlighting.pressed else {
                panic!("I support only fixed highlight.");
            };
            let Some(HighlightKind::<ColorMaterial>::Fixed(pressed_highlight)) = &highlighting_mats.node_materials_blocked.pressed else {
                panic!("conf support only fixed highlight.");
            };
            if current_highlight != pressed_highlight {
                *q_visibility.get_mut(eye_catcher.0).unwrap() = Visibility::Hidden;
                *highlighting = highlighting_mats.node_materials_blocked.clone();
            }

            if manual.is_some() && self_status.is_blocked {
                *q_visibility.get_mut(eye_catcher.0).unwrap() = Visibility::Hidden;
            }
        } else {
            *q_visibility.get_mut(eye_catcher.0).unwrap() = Visibility::Visible;
            let Some(HighlightKind::<ColorMaterial>::Fixed(current_highlight)) = &highlighting.pressed else {
                panic!("I support only fixed highlight.");
            };
            let Some(HighlightKind::<ColorMaterial>::Fixed(normal_highlight)) = &highlighting_mats.node_materials_normal.pressed else {
                panic!("conf support only fixed highlight.");
            };
            if current_highlight != normal_highlight {
                *q_visibility.get_mut(eye_catcher.0).unwrap() = Visibility::Hidden;
                *highlighting = highlighting_mats.node_materials_normal.clone();
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

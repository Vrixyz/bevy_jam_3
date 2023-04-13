use crate::*;
use bevy::prelude::*;
use bevy_mod_picking::Highlighting;

#[derive(Resource)]
pub struct HighlightingMaterials {
    pub node_materials_normal: Highlighting<ColorMaterial>,
    pub node_materials_blocked: Highlighting<ColorMaterial>,
}

pub fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let mat_initial = materials.add(ColorMaterial::from(Color::WHITE));
    let mat_initial_blocked = materials.add(ColorMaterial::from(Color::ANTIQUE_WHITE));
    let res = HighlightingMaterials {
        node_materials_normal: Highlighting {
            initial: mat_initial.clone(),
            hovered: Some(materials.add(ColorMaterial::from(Color::GRAY))),
            pressed: Some(materials.add(ColorMaterial::from(Color::GREEN))),
            selected: Some(mat_initial.clone()),
        },
        node_materials_blocked: Highlighting {
            initial: mat_initial_blocked.clone(),
            hovered: Some(materials.add(ColorMaterial::from(Color::DARK_GRAY))),
            pressed: Some(materials.add(ColorMaterial::from(Color::DARK_GREEN))),
            selected: Some(mat_initial_blocked.clone()),
        },
    };
    commands.insert_resource(res);
}

pub fn button_react(
    mut events: EventReader<PickingEvent>,
    mut events_writer: EventWriter<NewNodeEvent>,
    mut events_reset_writer: EventWriter<PropagateResetManualButtons>,
    mut q_timer: Query<(&mut Progress, &InheritedBlockStatus, &mut NodeCurrencyGain)>,
    mut currencies: ResMut<Currency>,
) {
    for event in events.iter() {
        if let PickingEvent::Clicked(e) = event {
            let Ok((mut p, status, mut gain)) = q_timer.get_mut(*e) else {
                continue;
            };
            if !status.is_blocked && p.timer.finished() {
                //dbg!("GAIN!");
                currencies.amount += 1;
                let new_time_duration = currencies.amount as f32 * TIMER_GAIN_MULT
                    + TIMER_GAIN_MULT_PER_LEVEL * gain.0 as f32;
                p.timer
                    .set_duration(Duration::from_secs_f32(new_time_duration));
                p.timer.reset();
                gain.0 += 1;
                events_writer.send(NewNodeEvent((*e, currencies.amount)));
                events_reset_writer.send(PropagateResetManualButtons(*e));
            } else {
                dbg!("NOT READY");
            }
        }
    }
}

#[derive(Component)]
pub struct AutoClick(pub Entity);

pub fn auto_click(
    mut events: EventWriter<PickingEvent>,
    q_autoclick: Query<(&Visibility, &AutoClick)>,
    mut q_interact: Query<&mut Interaction>,
) {
    for (v, auto_click) in q_autoclick.iter() {
        if v == Visibility::Visible {
            events.send(PickingEvent::Clicked(auto_click.0))
        }
    }
}

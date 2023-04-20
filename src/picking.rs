use crate::*;

use bevy::utils::Uuid;

use bevy_picking_core::backend::PickData;
use bevy_picking_core::events::PointerEvent;
use bevy_picking_highlight::*;

#[derive(Resource)]
pub struct HighlightingMaterials {
    pub mat_normal: Handle<ColorMaterial>,
    pub mat_blocked: Handle<ColorMaterial>,
    pub node_materials_normal: HighlightOverride<ColorMaterial>,
    pub node_materials_blocked: HighlightOverride<ColorMaterial>,
}

pub fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let mat_initial = materials.add(ColorMaterial::from(Color::WHITE));
    let mat_initial_blocked = materials.add(ColorMaterial::from(Color::ANTIQUE_WHITE));
    let res = HighlightingMaterials {
        mat_normal: mat_initial.clone(),
        mat_blocked: mat_initial_blocked.clone(),
        node_materials_normal: HighlightOverride {
            hovered: Some(HighlightKind::Fixed(
                materials.add(ColorMaterial::from(Color::GRAY)),
            )),
            pressed: Some(HighlightKind::Fixed(
                materials.add(ColorMaterial::from(Color::GREEN)),
            )),
            selected: Some(HighlightKind::Fixed(mat_initial.clone())),
        },
        node_materials_blocked: HighlightOverride {
            hovered: Some(HighlightKind::Fixed(
                materials.add(ColorMaterial::from(Color::DARK_GRAY)),
            )),
            pressed: Some(HighlightKind::Fixed(
                materials.add(ColorMaterial::from(Color::DARK_GREEN)),
            )),
            selected: Some(HighlightKind::Fixed(mat_initial_blocked.clone())),
        },
    };
    commands.insert_resource(res);
}

pub fn button_react(
    mut events: EventReader<PointerEvent<Down>>,
    mut events_writer: EventWriter<NewNodeEvent>,
    mut events_reset_writer: EventWriter<PropagateResetManualButtons>,
    mut q_timer: Query<(&mut Progress, &InheritedBlockStatus, &mut NodeCurrencyGain)>,
    mut currencies: ResMut<Currency>,
) {
    for event in events.iter() {
        let e = event.target();
        let Ok((mut p, status, mut gain)) = q_timer.get_mut(e) else {
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
            events_writer.send(NewNodeEvent((e, currencies.amount)));
            events_reset_writer.send(PropagateResetManualButtons(e));
        } else {
            dbg!("NOT READY");
        }
    }
}

#[derive(Component)]
pub struct AutoClick(pub Entity);

pub fn auto_click(
    mut events: EventWriter<PointerEvent<Down>>,
    q_autoclick: Query<(&Visibility, &AutoClick)>,
) {
    for (v, auto_click) in q_autoclick.iter() {
        if v == Visibility::Visible {
            events.send(PointerEvent::<Down>::new(
                &PointerId::Custom(Uuid::new_v4()),
                &auto_click.0,
                Down {
                    button: PointerButton::Primary,
                    pick_data: PickData {
                        camera: Entity::PLACEHOLDER,
                        depth: 1f32,
                        position: None,
                        normal: None,
                    },
                },
            ))
        }
    }
}

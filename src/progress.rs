use bevy::prelude::*;

use crate::*;

#[derive(Component, Debug)]
pub struct Progress {
    pub timer: Timer,
}

pub fn update_progress_timer(
    time: Res<Time>,
    mut q_timer: Query<(
        &mut Progress,
        &InheritedBlockStatus,
        Option<&NodeManualBlockToggle>,
    )>,
) {
    for (mut t, status, manual) in q_timer.iter_mut() {
        if let Some(manual) = manual {
            if manual.is_blocked && status.is_blocked {
                continue;
            }
        } else if status.is_blocked {
            t.timer.tick(time.delta().div_f32(3f32));
            continue;
        }
        t.timer.tick(time.delta());
    }
}

pub fn update_progress_manual_auto_block(
    currencies: Res<Currency>,
    mut events_writer: EventWriter<NewNodeEvent>,
    mut q_timer: Query<(Entity, &Progress, &mut NodeManualBlockToggle)>,
) {
    for (e, t, mut status) in q_timer.iter_mut() {
        if t.timer.just_finished() {
            status.is_blocked = true;
            events_writer.send(NewNodeEvent((e, currencies.amount)));
        }
    }
}

pub fn update_progress_text(
    mut q_texts: Query<(&mut Text, &ButtonRef)>,
    q_timer: Query<
        (
            &Progress,
            &InheritedBlockStatus,
            &SelfBlockStatus,
            Option<&NodeManualBlockToggle>,
        ),
        Or<(Changed<Progress>, Changed<InheritedBlockStatus>)>,
    >,
) {
    for (mut t, b) in q_texts.iter_mut() {
        match q_timer.get(b.0) {
            Err(_) => {}
            Ok((p, status, self_status, manual_toggle)) => {
                let block_status = if status.is_blocked { " (blocked)" } else { "" };
                if p.timer.finished() {
                    let text = if manual_toggle.is_some() {
                        if self_status.is_blocked {
                            "Unblock"
                        } else {
                            "Block"
                        }
                        .into()
                    } else {
                        format!("Gain!{}", block_status)
                    };
                    t.sections[0].value = text;
                } else {
                    t.sections[0].value =
                        format!("{:.0}s{}", p.timer.remaining_secs().ceil(), block_status);
                }
            }
        }
    }
}

pub fn update_progress_material(
    mut commands: Commands,
    q_timer_materials: Query<&TimerMaterials>,
    q_mat: Query<&ButtonRef, With<Handle<TimerMaterial>>>,
    q_timer: Query<
        (
            &Progress,
            &InheritedBlockStatus,
            &SelfBlockStatus,
            Option<&NodeManualBlockToggle>,
        ),
        Or<(Changed<Progress>, Changed<InheritedBlockStatus>)>,
    >,
) {
    let timer_materials = q_timer_materials.single();
    for (b) in q_mat.iter() {
        match q_timer.get(b.0) {
            Err(_) => {}
            Ok((p, status, self_status, manual_toggle)) => {
                if p.timer.finished() {
                    // nope
                } else {
                    commands
                        .entity(b.0)
                        .insert(timer_materials.get_material(p.timer.percent()));
                }
            }
        }
    }
}

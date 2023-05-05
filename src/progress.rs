use bevy::prelude::*;

use crate::*;

#[derive(Component, Debug)]
pub struct Progress {
    pub timer: Timer,
}

#[derive(Component, Debug)]
pub struct NodeTextValidate {
    pub text: String,
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
            t.timer.tick(time.delta().div_f32(20f32));
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
            events_writer.send(NewNodeEvent {
                entity: e,
                currencies_on_click: currencies.amount,
            });
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
            Option<&NodeTextValidate>,
        ),
        Or<(Changed<Progress>, Changed<InheritedBlockStatus>)>,
    >,
) {
    for (mut t, b) in q_texts.iter_mut() {
        match q_timer.get(b.0) {
            Err(_) => {
                t.sections[0].value = "error".into();
            }
            Ok((p, status, self_status, manual_toggle, text_validate)) => {
                if status.is_blocked {
                    t.sections[0].value = "Blocked".to_string();
                    continue;
                }
                if p.timer.finished() {
                    let text = if manual_toggle.is_some() {
                        if self_status.is_blocked {
                            "Unblock"
                        } else {
                            "Block"
                        }
                    } else {
                        text_validate.map_or("???", |t| &t.text)
                    };
                    t.sections[0].value = text.to_string();
                } else {
                    t.sections[0].value = format!("{:.0}s", p.timer.remaining_secs().ceil());
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
    for b in q_mat.iter() {
        match q_timer.get(b.0) {
            Err(_) => {}
            Ok((p, _status, _self_status, _manual_toggle)) => {
                if p.timer.finished() {
                    // nope
                    if p.timer.just_finished() {
                        commands
                            .entity(b.0)
                            .insert(timer_materials.get_material(1f32));
                    }
                } else {
                    commands
                        .entity(b.0)
                        .insert(timer_materials.get_material(p.timer.percent()));
                }
            }
        }
    }
}

// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::{ops::ControlFlow, time::Duration};

use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::WindowResolution,
};
use bevy_easings::EasingsPlugin;
use bevy_mod_picking::{
    DebugEventsPickingPlugin, DefaultPickingPlugins, Highlighting, PickableBundle,
    PickingCameraBundle, PickingEvent,
};
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use idle_gains::Currency;
use new_node::*;
use poisson::Poisson;
use progress::Progress;
use rand::{thread_rng, Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use status_visual::update_status_visual;

mod idle_gains;
mod new_node;
mod poisson;
mod progress;
mod status_visual;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                //resolution: WindowResolution::new(640., 640.).with_scale_factor_override(1.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(DefaultPickingPlugins) // <- Adds picking, interaction, and highlighting
        //.add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging.
        .add_plugin(PanCamPlugin::default())
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(EasingsPlugin)
        .add_event::<NewNodeEvent>()
        .add_event::<PropagateResetManualButtons>()
        .init_resource::<Currency>()
        .add_startup_system(setup)
        .add_system(update_progress_timer)
        .add_system(update_progress_manual_auto_block)
        .add_system(button_react)
        .add_system(reset_manual_button_timers.after(button_react))
        .add_system(button_manual_toggle_block_react)
        .add_system(new_button)
        .add_system(check_self_block)
        .add_system(update_inherited_block_status.after(check_self_block))
        .add_system(update_progress_text.after(update_inherited_block_status))
        .add_system(draw_relations.before(new_button))
        .add_system(update_status_visual.after(update_inherited_block_status))
        .run();
}

#[derive(Component)]
pub struct ButtonRef(pub Entity);

// node behaviours

#[derive(Component)]
pub struct NodeManualBlockToggle {
    pub is_blocked: bool,
}
#[derive(Component)]
pub struct NodeCurrencyGain;
//

/// To know which nodes are blocking our behaviour.
#[derive(Component)]
pub struct Blockers {
    pub entities: Vec<Entity>,
}

/// To know which nodes to block.
#[derive(Component)]
pub struct ToBlock {
    pub entities: Vec<Entity>,
}

#[derive(Component)]
pub struct InheritedBlockStatus {
    pub is_blocked: bool,
}
#[derive(Component)]
pub struct SelfBlockStatus {
    pub is_blocked: bool,
}

#[derive(Resource)]
pub struct MapAssets {
    pub font: Handle<Font>,
    pub text_style: TextStyle,
    pub mesh_gain: Mesh2dHandle,
    pub mesh_blocker: Mesh2dHandle,
    pub eye_catcher_mesh: Mesh2dHandle,
    pub eye_catcher_material: Handle<ColorMaterial>,
    pub node_materials_normal: Highlighting<ColorMaterial>,
    pub node_materials_blocked: Highlighting<ColorMaterial>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    currencies: ResMut<Currency>,
) {
    commands.insert_resource(RandomForMap::default());
    // 2d camera
    commands.spawn((
        Camera2dBundle::default(),
        PickingCameraBundle::default(),
        PanCam::default(),
    ));

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mat_initial = materials.add(ColorMaterial::from(Color::WHITE));
    let mat_initial_blocked = materials.add(ColorMaterial::from(Color::ANTIQUE_WHITE));
    let map_assets = MapAssets {
        font: font.clone(),
        text_style: TextStyle {
            font,
            font_size: 30.0,
            color: Color::BLACK,
        },
        mesh_gain: meshes.add(Mesh::from(shape::Circle::default())).into(),
        mesh_blocker: meshes.add(Mesh::from(shape::Quad::default())).into(),
        eye_catcher_mesh: meshes
            .add(Mesh::from(shape::RegularPolygon::new(0.5f32, 6)))
            .into(),
        eye_catcher_material: materials.add(ColorMaterial::from(Color::YELLOW_GREEN)),
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

    let button_entity = create_node(
        &mut commands,
        map_assets.mesh_gain.clone(),
        &map_assets,
        Vec2::ZERO,
        currencies.amount as f32,
    );
    commands.entity(button_entity).insert(NodeCurrencyGain);
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("translation", map_assets.text_style.clone())
                .with_alignment(TextAlignment::Center),
            transform: Transform::default().with_translation(Vec2::ZERO.extend(1f32)),
            ..default()
        },
        ButtonRef(button_entity),
    ));

    commands.insert_resource(map_assets);
}

fn update_progress_timer(
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
            continue;
        }
        t.timer.tick(time.delta());
    }
}
fn update_progress_manual_auto_block(
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

fn update_progress_text(
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

fn button_react(
    mut events: EventReader<PickingEvent>,
    mut events_writer: EventWriter<NewNodeEvent>,
    mut events_reset_writer: EventWriter<PropagateResetManualButtons>,
    mut q_timer: Query<(&mut Progress, &InheritedBlockStatus), With<NodeCurrencyGain>>,
    mut currencies: ResMut<Currency>,
) {
    for event in events.iter() {
        if let PickingEvent::Clicked(e) = event {
            let Ok((mut p, status)) = q_timer.get_mut(*e) else {
                continue;
            };
            if !status.is_blocked && p.timer.finished() {
                dbg!("GAIN!");
                currencies.amount += 1;
                let new_time_duration = p.timer.duration().as_secs_f32() + currencies.amount as f32;
                let new_time_duration = currencies.amount as f32 * TIMER_GAIN_MULT;
                p.timer
                    .set_duration(Duration::from_secs_f32(new_time_duration));
                p.timer.reset();
                events_writer.send(NewNodeEvent((*e, currencies.amount)));
                events_reset_writer.send(PropagateResetManualButtons(*e));
            } else {
                dbg!("NOT READY");
            }
        }
    }
}

fn button_manual_toggle_block_react(
    mut events: EventReader<PickingEvent>,
    mut q_nodes: Query<(&Progress, &mut NodeManualBlockToggle, &InheritedBlockStatus)>,
) {
    for event in events.iter() {
        if let PickingEvent::Clicked(e) = event {
            let Ok((p, mut node, status)) = q_nodes.get_mut(*e) else {
                continue;
            };
            if status.is_blocked {
                dbg!("node is blocked");
                continue;
            }
            if p.timer.finished() {
                dbg!("toggle block!");
                node.is_blocked = !node.is_blocked;
            } else {
                dbg!("NOT READY");
            }
        }
    }
}

fn check_self_block(
    mut q_nodes: Query<
        (&Progress, &mut NodeManualBlockToggle, &mut SelfBlockStatus),
        Or<(Changed<Progress>, Changed<NodeManualBlockToggle>)>,
    >,
) {
    for (p, manual, mut status) in q_nodes.iter_mut() {
        status.is_blocked = !p.timer.finished() || manual.is_blocked;
    }
}

fn update_inherited_block_status(
    mut commands: Commands,
    map_assets: Res<MapAssets>,
    mut q_blockers: Query<(&Blockers, &ToBlock, Entity), With<BaseNode>>,
    mut q_blockStatus: Query<(&mut InheritedBlockStatus, &SelfBlockStatus), With<BaseNode>>,
) {
    for (mut inherited_status, _) in q_blockStatus.iter_mut() {
        inherited_status.is_blocked = false;
    }
    for (blockers, to_blocks, e) in q_blockers.iter() {
        if blockers.entities.is_empty() {
            recurse_block(&q_blockers, &mut q_blockStatus, e, false, to_blocks);
        }
    }
}

fn recurse_block(
    q_blockers: &Query<(&Blockers, &ToBlock, Entity), With<BaseNode>>,
    q_block_status: &mut Query<(&mut InheritedBlockStatus, &SelfBlockStatus), With<BaseNode>>,
    e: Entity,
    is_blocked: bool,
    to_blocks: &ToBlock,
) {
    let (mut inherited_status, self_status) = q_block_status
        .get_mut(e)
        .expect("all nodes should have a status");
    if inherited_status.is_blocked {
        // a previous check has blocked all hierarchy.
        return;
    }
    if is_blocked {
        inherited_status.is_blocked = true;
    }
    let status = inherited_status.is_blocked || self_status.is_blocked;
    for to_block in to_blocks.entities.iter() {
        let blockers = q_blockers
            .get(*to_block)
            .expect("blocker cannot be destroyed");
        recurse_block(q_blockers, q_block_status, *to_block, status, blockers.1);
    }
}

pub struct PropagateResetManualButtons(pub Entity);

fn reset_manual_button_timers(
    currencies: Res<Currency>,
    mut events: EventReader<PropagateResetManualButtons>,
    mut q_blockers: Query<(&Blockers, Entity), With<BaseNode>>,
    mut q_manual_node: Query<
        (
            &mut NodeManualBlockToggle,
            &mut SelfBlockStatus,
            &mut Progress,
        ),
        With<BaseNode>,
    >,
) {
    for e in events.iter() {
        recurse_reset_manual(&e.0, &currencies, &mut q_manual_node, &q_blockers);
    }
}

fn recurse_reset_manual(
    e: &Entity,
    currencies: &Res<Currency>,
    q_manual_node: &mut Query<
        (
            &mut NodeManualBlockToggle,
            &mut SelfBlockStatus,
            &mut Progress,
        ),
        With<BaseNode>,
    >,
    q_blockers: &Query<(&Blockers, Entity), With<BaseNode>>,
) {
    if let Ok((mut manual, mut self_status, mut progress)) = q_manual_node.get_mut(*e) {
        manual.is_blocked = true;
        self_status.is_blocked = true;
        progress
            .timer
            .set_duration(Duration::from_secs_f32(TIMER_RESET_BLOCKER_FIXED));
        progress.timer.reset();
    }
    let Ok(blockers) = q_blockers.get(*e) else {
        return;
    };
    for child in blockers.0.entities.iter() {
        recurse_reset_manual(child, currencies, q_manual_node, q_blockers);
    }
}

fn draw_relations(
    mut commands: Commands,
    mut lines: ResMut<DebugLines>,
    map_assets: Res<MapAssets>,
    mut q_to_block: Query<(&ToBlock, Entity), With<BaseNode>>,
    mut q_blockers: Query<(&Transform, &Blockers, Entity), With<BaseNode>>,
    mut q_blockStatus: Query<(&Transform, &InheritedBlockStatus, &SelfBlockStatus), With<BaseNode>>,
) {
    for (to_blocks, e) in q_to_block.iter() {
        if to_blocks.entities.is_empty() {
            recurse_draw_relations(&mut lines, &q_blockers, &q_blockStatus, e);
        }
    }
}

fn recurse_draw_relations(
    mut lines: &mut ResMut<DebugLines>,
    q_blockers: &Query<(&Transform, &Blockers, Entity), With<BaseNode>>,
    q_block_status: &Query<(&Transform, &InheritedBlockStatus, &SelfBlockStatus), With<BaseNode>>,
    e: Entity,
) {
    let (t, mut inherited_status, self_status) = q_block_status
        .get(e)
        .expect("all nodes should have a status");

    let status = inherited_status.is_blocked;
    if let Ok(blockers) = q_blockers.get(e) {
        for new_blocker in blockers.1.entities.iter() {
            lines.line_colored(
                t.translation,
                q_blockers
                    .get(*new_blocker)
                    .expect("all nodes have a transform")
                    .0
                    .translation,
                0f32,
                if inherited_status.is_blocked {
                    Color::RED
                } else {
                    Color::GREEN
                },
            );
            recurse_draw_relations(lines, q_blockers, q_block_status, *new_blocker);
        }
    }
}

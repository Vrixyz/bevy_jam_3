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
};
use bevy_mod_picking::{
    DebugEventsPickingPlugin, DefaultPickingPlugins, PickableBundle, PickingCameraBundle,
    PickingEvent,
};
use bevy_pancam::{PanCam, PanCamPlugin};
use poisson::Poisson;
use progress::Progress;
use rand::{thread_rng, Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

mod idle_gains;
mod poisson;
mod progress;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins) // <- Adds picking, interaction, and highlighting
        //.add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging.
        .add_plugin(PanCamPlugin::default())
        .add_event::<CurrencyGainEvent>()
        .init_resource::<Currency>()
        .add_startup_system(setup)
        .add_system(update_progress_timer)
        .add_system(update_progress_manual_auto_block)
        .add_system(button_react)
        .add_system(button_manual_toggle_block_react)
        .add_system(new_button)
        .add_system(check_self_block)
        .add_system(update_inherited_block_status.after(check_self_block))
        .add_system(update_progress_text.after(update_inherited_block_status))
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
pub struct BaseNode;

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
    pub mesh: Mesh2dHandle,
    pub material: Handle<ColorMaterial>,
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
    let map_assets = MapAssets {
        font: font.clone(),
        text_style: TextStyle {
            font,
            font_size: 30.0,
            color: Color::WHITE,
        },
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
    };

    let button_entity = create_node(&mut commands, &map_assets, Vec2::ZERO, currencies.value);
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

fn create_node(
    commands: &mut Commands,
    map_assets: &MapAssets,
    pos: Vec2,
    duration: f32,
) -> Entity {
    let ent = commands.spawn((
        MaterialMesh2dBundle {
            mesh: map_assets.mesh.clone(),
            transform: Transform::default()
                .with_translation(pos.extend(0f32))
                .with_scale(Vec3::splat(128.)),
            material: map_assets.material.clone(),
            ..default()
        },
        PickableBundle::default(),
        Progress {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        },
        BaseNode,
        InheritedBlockStatus { is_blocked: false },
        SelfBlockStatus { is_blocked: false },
        Blockers { entities: vec![] },
        ToBlock { entities: vec![] },
    ));
    ent.id()
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
        if !status.is_blocked || manual.is_some() {
            t.timer.tick(time.delta());
        }
    }
}
fn update_progress_manual_auto_block(mut q_timer: Query<(&Progress, &mut NodeManualBlockToggle)>) {
    for (t, mut status) in q_timer.iter_mut() {
        if t.timer.just_finished() {
            status.is_blocked = true;
        }
    }
}

fn update_progress_text(
    mut q_texts: Query<(&mut Text, &ButtonRef)>,
    q_timer: Query<
        (
            &Progress,
            &InheritedBlockStatus,
            Option<&NodeManualBlockToggle>,
        ),
        Or<(Changed<Progress>, Changed<InheritedBlockStatus>)>,
    >,
) {
    for (mut t, b) in q_texts.iter_mut() {
        match q_timer.get(b.0) {
            Err(_) => {}
            Ok((p, status, manual_toggle)) => {
                let block_status = if status.is_blocked { " (blocked)" } else { "" };
                if p.timer.finished() {
                    let text = if manual_toggle.is_some() {
                        if status.is_blocked {
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

#[derive(Resource, Default)]
struct Currency {
    value: f32,
}
fn button_react(
    mut events: EventReader<PickingEvent>,
    mut events_writer: EventWriter<CurrencyGainEvent>,
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
                currencies.value += 1f32;
                let new_time_duration = p.timer.duration().as_secs_f32() + currencies.value;
                let new_time_duration = currencies.value * 2f32;
                p.timer
                    .set_duration(Duration::from_secs_f32(new_time_duration));
                p.timer.reset();
                events_writer.send(CurrencyGainEvent((*e, currencies.value)));
            } else {
                dbg!("NOT READY");
            }
        }
    }
}

fn button_manual_toggle_block_react(
    mut events: EventReader<PickingEvent>,
    mut q_nodes: Query<(&Progress, &mut NodeManualBlockToggle)>,
) {
    for event in events.iter() {
        if let PickingEvent::Clicked(e) = event {
            let Ok((p, mut node)) = q_nodes.get_mut(*e) else {
                continue;
            };
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

struct CurrencyGainEvent(pub (Entity, f32));

#[derive(Resource)]
pub struct RandomForMap {
    pub(crate) random: ChaCha20Rng,
    pub(crate) seed: u64,
}

impl Default for RandomForMap {
    fn default() -> Self {
        let seed = thread_rng().gen::<u64>();
        Self {
            random: ChaCha20Rng::seed_from_u64(seed),
            seed,
        }
    }
}

fn new_button(
    mut commands: Commands,
    map_assets: Res<MapAssets>,
    mut random_map: ResMut<RandomForMap>,
    mut events: EventReader<CurrencyGainEvent>,
    q_nodes: Query<(&Transform, Entity), With<BaseNode>>,
    mut q_blockers: Query<(&mut Blockers, Entity), With<BaseNode>>,
    currencies: Res<Currency>,
) {
    for event in events.iter() {
        let positions: Vec<_> = q_nodes
            .iter()
            .map(|(t, e)| {
                let pos = t.translation.truncate();
                ((pos.x, pos.y), e)
            })
            .collect();
        let existing_points: Vec<_> = positions.into_iter().map(|(p, _)| p).collect();
        let poisson = Poisson::new();
        let entity_from = event.0 .0;
        let pos = q_nodes
            .get(entity_from)
            .expect("entity in event gain currency should be valid")
            .0
            .translation
            .truncate();
        match poisson.compute_new_position(
            &existing_points,
            &(pos.x, pos.y),
            200f32,
            10,
            &mut random_map.random,
        ) {
            None => {}
            Some(pos) => {
                let node = create_node(
                    &mut commands,
                    &map_assets,
                    Vec2::new(pos.0, pos.1),
                    event.0 .1 * 2f32,
                );
                if (currencies.value as i32) % 2 == 0 {
                    commands
                        .entity(node)
                        .insert(NodeManualBlockToggle { is_blocked: false })
                        .insert(SelfBlockStatus { is_blocked: false });
                } else {
                    commands.entity(node).insert(NodeCurrencyGain);
                }

                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section("", map_assets.text_style.clone())
                            .with_alignment(TextAlignment::Center),
                        transform: Transform::default()
                            .with_translation(Vec2::new(pos.0, pos.1).extend(1f32)),
                        ..default()
                    },
                    ButtonRef(node),
                ));

                commands.entity(node).insert(ToBlock {
                    entities: vec![entity_from],
                });

                if let Ok(mut blockers) = q_blockers.get_mut(entity_from) {
                    blockers.0.entities.push(node);
                } else {
                    commands.entity(entity_from).insert(Blockers {
                        entities: vec![node],
                    });
                }
                break;
            }
        }
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
    let is_blocked = false;
    for (blockers, to_blocks, e) in q_blockers.iter() {
        if blockers.entities.is_empty() {
            recurse_block(&q_blockers, &mut q_blockStatus, e, is_blocked, to_blocks);
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
    if is_blocked || self_status.is_blocked {
        inherited_status.is_blocked = true;
    }
    let status = self_status.is_blocked;
    for to_block in to_blocks.entities.iter() {
        let blockers = q_blockers
            .get(*to_block)
            .expect("blocker cannot be destroyed");
        recurse_block(q_blockers, q_block_status, *to_block, status, blockers.1);
    }
}

use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_easings::{Ease, EaseMethod, EasingType};
use bevy_mod_picking::PickableBundle;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::thread_rng;
use rand_chacha::ChaCha20Rng;

use crate::persisted_game::NodeType;
use crate::{picking::AutoClick, picking::HighlightingMaterials, picking_aabb::HalfExtents, *};

pub const TIMER_BLOCKER_MULT: f32 = 0.5f32; // / 10000f32;
pub const TIMER_RESET_BLOCKER_FIXED: f32 = 0.5f32; // / 1000f32;
pub const TIMER_GAIN_MULT: f32 = 0.3f32; // / 100000f32;
pub const TIMER_GAIN_MULT_PER_LEVEL: f32 = 2f32; // / 10000f32;
pub const TIMER_SAVE_BASE: f32 = 5f32; // / 10000f32;
pub const TIMER_SAVE_MULT_PER_LEVEL: f32 = 5f32; // / 10000f32;
pub const TIMER_SAVE_ADD_MULT_PER_CURRENCY: f32 = 0.5f32; // / 10000f32;

pub struct NewNodeEvent {
    pub entity: Entity,
    pub currencies_on_click: i32,
}

#[derive(Component)]
pub struct BaseNode;
#[derive(Component)]
pub struct EyeCatcher(pub Entity);

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

pub fn create_node(
    commands: &mut Commands,
    mesh: Mesh2dHandle,
    map_assets: &MapAssets,
    highlights: &HighlightingMaterials,
    pos: Vec2,
    duration: f32,
    elapsed_time: f32,
) -> Entity {
    let eye_catcher = commands.spawn(bundle_eye_catcher(map_assets, pos)).id();
    let ent = commands
        .spawn(bundle_node(
            mesh,
            pos,
            highlights,
            eye_catcher,
            duration,
            elapsed_time,
        ))
        .id();
    commands.entity(eye_catcher).insert(AutoClick(ent));
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: map_assets.mesh_gain.clone(),
            material: Handle::<TimerMaterial>::default(),
            transform: Transform::default().with_translation(pos.extend(11f32)),
            ..default()
        },
        ButtonRef(ent),
    ));
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("", map_assets.text_style.clone())
                .with_alignment(TextAlignment::Center),
            transform: Transform::default().with_translation(pos.extend(1f32)),
            ..default()
        },
        ButtonRef(ent),
    ));
    ent
}

pub fn insert_node(
    commands: &mut Commands,
    mesh: Mesh2dHandle,
    map_assets: &MapAssets,
    highlights: &HighlightingMaterials,
    pos: Vec2,
    duration: f32,
    elapsed_time: f32,
    entity: Entity,
) -> Entity {
    let eye_catcher = commands.spawn(bundle_eye_catcher(map_assets, pos)).id();
    let ent = commands
        .entity(entity)
        .insert(bundle_node(
            mesh,
            pos,
            highlights,
            eye_catcher,
            duration,
            elapsed_time,
        ))
        .id();
    commands.entity(eye_catcher).insert(AutoClick(ent));
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: map_assets.mesh_gain.clone(),
            material: Handle::<TimerMaterial>::default(),
            transform: Transform::default().with_translation(pos.extend(11f32)),
            ..default()
        },
        ButtonRef(ent),
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("", map_assets.text_style.clone())
                .with_alignment(TextAlignment::Center),
            transform: Transform::default().with_translation(pos.extend(1f32)),
            ..default()
        },
        ButtonRef(ent),
    ));
    ent
}

fn bundle_node(
    mesh: Mesh2dHandle,
    pos: Vec2,
    highlights: &HighlightingMaterials,
    eye_catcher: Entity,
    duration: f32,
    elapsed_time: f32,
) -> (
    MaterialMesh2dBundle<ColorMaterial>,
    EyeCatcher,
    PickableBundle,
    Progress,
    BaseNode,
    InheritedBlockStatus,
    SelfBlockStatus,
    Blockers,
    ToBlock,
    HalfExtents,
    HighlightOverride<ColorMaterial>,
) {
    (
        MaterialMesh2dBundle {
            mesh: mesh.clone(),
            transform: Transform::default()
                .with_translation(pos.extend(1f32))
                .with_scale(Vec3::splat(128.)),
            material: highlights.mat_normal.clone(),
            ..default()
        },
        EyeCatcher(eye_catcher),
        PickableBundle::default(),
        Progress {
            timer: {
                let mut timer = Timer::from_seconds(duration, TimerMode::Once);
                timer.set_elapsed(Duration::from_secs_f32(elapsed_time));
                timer
            },
        },
        BaseNode,
        InheritedBlockStatus { is_blocked: false },
        SelfBlockStatus { is_blocked: false },
        Blockers { entities: vec![] },
        ToBlock { entities: vec![] },
        HalfExtents(Vec2::splat(128f32 / 2f32)),
        highlights.node_materials_normal.clone(),
    )
}

fn bundle_eye_catcher(
    map_assets: &MapAssets,
    pos: Vec2,
) -> (
    MaterialMesh2dBundle<ColorMaterial>,
    bevy_easings::EasingComponent<Transform>,
) {
    (
        MaterialMesh2dBundle {
            mesh: map_assets.eye_catcher_mesh.clone(),
            material: map_assets.eye_catcher_material.clone(),
            visibility: Visibility::Hidden,
            ..default()
        },
        Transform {
            translation: pos.extend(0f32),
            scale: Vec3::splat(160.),
            rotation: Quat::from_axis_angle(Vec3::Z, 0f32),
            ..default()
        }
        .ease_to(
            Transform {
                translation: pos.extend(0f32),
                scale: Vec3::splat(160.),
                rotation: Quat::from_axis_angle(Vec3::Z, 6f32 / 12f32 * std::f32::consts::TAU),
                ..default()
            },
            EaseMethod::Linear,
            EasingType::Loop {
                duration: std::time::Duration::from_millis(1000),
                pause: Some(std::time::Duration::from_millis(0)),
            },
        ),
    )
}

pub fn new_button(
    mut commands: Commands,
    map_assets: Res<MapAssets>,
    highlights: Res<HighlightingMaterials>,
    mut random_map: ResMut<RandomForMap>,
    mut events: EventReader<NewNodeEvent>,
    q_nodes: Query<(&Transform, Entity), With<BaseNode>>,
    mut q_blockers: Query<(&mut Blockers, Entity), With<BaseNode>>,
) {
    if events.is_empty() {
        return;
    }
    let positions: Vec<_> = q_nodes
        .iter()
        .map(|(t, e)| {
            let pos = t.translation.truncate();
            ((pos.x, pos.y), e)
        })
        .collect();
    let mut existing_points: Vec<_> = positions.into_iter().map(|(p, _)| p).collect();

    for event in events.iter() {
        let poisson = Poisson::new();
        let entity_from = event.entity;
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
                let choices = [
                    None,
                    Some(NodeType::Blocker { is_blocked: true }),
                    Some(NodeType::Save { level: 1 }),
                    Some(NodeType::Gain { level: 1 }),
                ];
                let weights = [
                    if event.currencies_on_click <= 1 {
                        0
                    } else {
                        30
                    },
                    50,
                    5,
                    20,
                ];
                let dist = WeightedIndex::new(&weights).unwrap();
                match choices[dist.sample(&mut random_map.random)] {
                    Some(NodeType::Blocker { is_blocked }) => {
                        create_blocker(
                            &mut commands,
                            &map_assets,
                            &highlights,
                            Vec2::new(pos.0, pos.1),
                            event.currencies_on_click as f32 * TIMER_BLOCKER_MULT,
                            &mut q_blockers,
                            entity_from,
                            is_blocked,
                        );
                        existing_points.push(pos);
                    }
                    Some(NodeType::Save { level }) => {
                        let node = create_node(
                            &mut commands,
                            map_assets.mesh_save.clone(),
                            &map_assets,
                            &highlights,
                            Vec2::new(pos.0, pos.1),
                            TIMER_SAVE_BASE
                                + (event.currencies_on_click as f32
                                    * TIMER_SAVE_ADD_MULT_PER_CURRENCY),
                            0f32,
                        );
                        commands.entity(node).insert(NodeTextValidate {
                            text: "Save".to_string(),
                        });
                        commands.entity(node).insert(NodeSave { level });
                        existing_points.push(pos);
                    }
                    Some(NodeType::Gain { level }) => {
                        let node = create_node(
                            &mut commands,
                            map_assets.mesh_gain.clone(),
                            &map_assets,
                            &highlights,
                            Vec2::new(pos.0, pos.1),
                            event.currencies_on_click as f32 * TIMER_GAIN_MULT
                                + TIMER_GAIN_MULT_PER_LEVEL,
                            0f32,
                        );

                        commands.entity(node).insert(NodeTextValidate {
                            text: "Gain!".to_string(),
                        });
                        commands.entity(node).insert(NodeCurrencyGain { level });
                        existing_points.push(pos);
                    }
                    None => {}
                }
            }
        }
    }
}

fn create_blocker(
    commands: &mut Commands,
    map_assets: &Res<MapAssets>,
    highlights: &Res<HighlightingMaterials>,
    pos: Vec2,
    duration: f32,
    q_blockers: &mut Query<(&mut Blockers, Entity), With<BaseNode>>,
    entity_from: Entity,
    is_blocked: bool,
) -> Entity {
    let node = create_node(
        commands,
        map_assets.mesh_blocker.clone(),
        map_assets,
        highlights,
        pos,
        duration,
        0f32,
    );
    commands
        .entity(node)
        .insert(NodeManualBlockToggle { is_blocked })
        .insert(SelfBlockStatus { is_blocked });
    if let Ok(mut blockers) = q_blockers.get_mut(entity_from) {
        blockers.0.entities.push(node);
    } else {
        commands.entity(entity_from).insert(Blockers {
            entities: vec![node],
        });
    }
    commands.entity(node).insert(ToBlock {
        entities: vec![entity_from],
    });
    node
}

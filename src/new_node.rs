use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_easings::{Ease, EaseFunction, EaseMethod, EasingType};
use bevy_mod_picking::PickableBundle;
use rand::thread_rng;
use rand_chacha::ChaCha20Rng;

use crate::{status_visual::AutoClick, *};

pub const TIMER_BLOCKER_MULT: f32 = 0.035f32 / 1000f32;
pub const TIMER_RESET_BLOCKER_FIXED: f32 = 0.05f32;
pub const TIMER_GAIN_MULT: f32 = 2.5f32 / 100000f32;

pub struct NewNodeEvent(pub (Entity, i32));

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
    pos: Vec2,
    duration: f32,
) -> Entity {
    let eye_catcher = commands.spawn(bundle_eye_catcher(map_assets, pos)).id();
    let ent = commands
        .spawn(bundle_node(mesh, pos, map_assets, eye_catcher, duration))
        .id();
    commands.entity(eye_catcher).insert(AutoClick(ent));
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("", map_assets.text_style.clone())
                .with_alignment(TextAlignment::Center),
            transform: Transform::default().with_translation(pos.extend(10f32)),
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
    pos: Vec2,
    duration: f32,
    entity: Entity,
) -> Entity {
    let eye_catcher = commands.spawn(bundle_eye_catcher(map_assets, pos)).id();
    let ent = commands
        .entity(entity)
        .insert(bundle_node(mesh, pos, map_assets, eye_catcher, duration))
        .id();
    commands.entity(eye_catcher).insert(AutoClick(ent));
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("", map_assets.text_style.clone())
                .with_alignment(TextAlignment::Center),
            transform: Transform::default().with_translation(pos.extend(10f32)),
            ..default()
        },
        ButtonRef(ent),
    ));
    ent
}

fn bundle_node(
    mesh: Mesh2dHandle,
    pos: Vec2,
    map_assets: &MapAssets,
    eye_catcher: Entity,
    duration: f32,
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
    Highlighting<ColorMaterial>,
) {
    (
        MaterialMesh2dBundle {
            mesh: mesh.clone(),
            transform: Transform::default()
                .with_translation(pos.extend(1f32))
                .with_scale(Vec3::splat(128.)),
            material: map_assets.node_materials_normal.initial.clone(),
            ..default()
        },
        EyeCatcher(eye_catcher),
        PickableBundle::default(),
        Progress {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        },
        BaseNode,
        InheritedBlockStatus { is_blocked: false },
        SelfBlockStatus { is_blocked: false },
        Blockers { entities: vec![] },
        ToBlock { entities: vec![] },
        map_assets.node_materials_normal.clone(),
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
    mut random_map: ResMut<RandomForMap>,
    mut events: EventReader<NewNodeEvent>,
    q_nodes: Query<(&Transform, Entity), With<BaseNode>>,
    mut q_blockers: Query<(&mut Blockers, Entity), With<BaseNode>>,
    currencies: Res<Currency>,
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
                let mut random_number = random_map.random.gen::<u32>() % 100;
                let chance_to_no_room = if currencies.amount <= 1 { 0 } else { 20 };
                if random_number < chance_to_no_room {
                    continue;
                }
                random_number -= chance_to_no_room;
                existing_points.push(pos);
                // 80 weight left?

                let node = if random_number < 55 {
                    let node = create_node(
                        &mut commands,
                        map_assets.mesh_blocker.clone(),
                        &map_assets,
                        Vec2::new(pos.0, pos.1),
                        event.0 .1 as f32 * TIMER_BLOCKER_MULT,
                    );
                    commands
                        .entity(node)
                        .insert(NodeManualBlockToggle { is_blocked: false })
                        .insert(SelfBlockStatus { is_blocked: false });
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
                } else {
                    let node = create_node(
                        &mut commands,
                        map_assets.mesh_gain.clone(),
                        &map_assets,
                        Vec2::new(pos.0, pos.1),
                        event.0 .1 as f32 * TIMER_GAIN_MULT,
                    );
                    commands.entity(node).insert(NodeCurrencyGain);
                    node
                };
            }
        }
    }
}

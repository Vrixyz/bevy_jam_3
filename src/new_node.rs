use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_mod_picking::PickableBundle;
use rand::thread_rng;
use rand_chacha::ChaCha20Rng;

use crate::*;

pub const TIMER_BLOCKER_MULT: f32 = 0.45f32;
pub const TIMER_RESET_BLOCKER_FIXED: f32 = 0.5f32;
pub const TIMER_GAIN_MULT: f32 = 2.5f32;

pub struct NewNodeEvent(pub (Entity, i32));

#[derive(Component)]
pub struct BaseNode;

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

pub(super) fn create_node(
    commands: &mut Commands,
    mesh: Mesh2dHandle,
    map_assets: &MapAssets,
    pos: Vec2,
    duration: f32,
) -> Entity {
    let eye_catcher = commands.spawn((MaterialMesh2dBundle {
        mesh: map_assets.eye_catcher_mesh.clone(),
        transform: Transform::default()
            .with_translation(pos.extend(0f32))
            .with_scale(Vec3::splat(160.)),
        material: map_assets.eye_catcher_material.clone(),
        ..default()
    },));
    let ent = commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh.clone(),
            transform: Transform::default()
                .with_translation(pos.extend(1f32))
                .with_scale(Vec3::splat(128.)),
            material: map_assets.node_materials_normal.initial.clone(),
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
        map_assets.node_materials_normal.clone(),
    ));
    ent.id()
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
                let chance_to_no_room = 30;
                if random_number < chance_to_no_room {
                    continue;
                }
                random_number -= chance_to_no_room;
                existing_points.push(pos);
                // 70 weight left

                let node = if random_number < 45 {
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

                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section("", map_assets.text_style.clone())
                            .with_alignment(TextAlignment::Center),
                        transform: Transform::default()
                            .with_translation(Vec2::new(pos.0, pos.1).extend(10f32)),
                        ..default()
                    },
                    ButtonRef(node),
                ));
            }
        }
    }
}
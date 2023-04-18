use bevy::{
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    new_node::{insert_node, BaseNode, TIMER_BLOCKER_MULT},
    picking::HighlightingMaterials,
    progress::Progress,
    Blockers, MapAssets, NodeCurrencyGain, NodeManualBlockToggle, SelfBlockStatus, ToBlock,
};

pub struct GameLoader;

impl Plugin for GameLoader {
    fn build(&self, app: &mut App) {
        app.register_type::<LoadingNode>();
        app.register_type::<LoadingNodes>();
        app.add_system(load_system);
        app.add_system(save.run_if(input_just_pressed(KeyCode::S)));
    }
}

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum NodeType {
    Gain { level: u32 },
    Blocker { is_blocked: bool },
}

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub struct SavedNode {
    pub pos: Vec2,
    pub node_type: NodeType,
    pub timer_seconds_duration: f32,
    pub timer_seconds_left: f32,
    pub to_block: Vec<usize>,
    pub blockers: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Save {
    pub last_tick_time_since2023: f32,
    pub nodes: Vec<SavedNode>,
}

pub fn load() -> Save {
    let data = Save {
        last_tick_time_since2023: 2f32,
        nodes: vec![
            SavedNode {
                pos: Vec2::default(),
                node_type: NodeType::Gain { level: 1 },
                timer_seconds_duration: 2.0,
                timer_seconds_left: 2.0,
                to_block: vec![],
                blockers: vec![1],
            },
            SavedNode {
                pos: Vec2::new(0f32, 230f32),
                node_type: NodeType::Blocker { is_blocked: true },
                timer_seconds_duration: 3.0,
                timer_seconds_left: 1.0,
                to_block: vec![0],
                blockers: vec![],
            },
        ],
    };
    let j = serde_json::to_string(&data).unwrap();
    println!("{}", j);

    let j = r#"{"last_tick_time_since2023":2.0,"nodes":[{"pos":[0.0,0.0],"node_type":{"Gain":{"level":2}},"timer_seconds_duration":3.2,"timer_seconds_left":3.044175,"to_block":[],"blockers":[4,7]},{"pos":[-44.699467,424.95117],"node_type":{"Gain":{"level":3}},"timer_seconds_duration":4.6,"timer_seconds_left":0.0,"to_block":[],"blockers":[5]},{"pos":[29.254093,610.78674],"node_type":{"Gain":{"level":3}},"timer_seconds_duration":5.5,"timer_seconds_left":5.364157,"to_block":[],"blockers":[9]},{"pos":[-147.21008,964.9743],"node_type":{"Gain":{"level":2}},"timer_seconds_duration":3.8000002,"timer_seconds_left":1.9001417,"to_block":[],"blockers":[]},{"pos":[0.0,230.0],"node_type":{"Blocker":{"is_blocked":true}},"timer_seconds_duration":0.5,"timer_seconds_left":0.0,"to_block":[0],"blockers":[8]},{"pos":[-181.68568,570.68646],"node_type":{"Blocker":{"is_blocked":false}},"timer_seconds_duration":0.4,"timer_seconds_left":0.0,"to_block":[1],"blockers":[6]},{"pos":[-193.33986,770.3566],"node_type":{"Blocker":{"is_blocked":false}},"timer_seconds_duration":0.6,"timer_seconds_left":0.0,"to_block":[5],"blockers":[]},{"pos":[-199.9941,2.5221765],"node_type":{"Blocker":{"is_blocked":true}},"timer_seconds_duration":0.8,"timer_seconds_left":0.0,"to_block":[0],"blockers":[10]},{"pos":[199.91447,223.81909],"node_type":{"Blocker":{"is_blocked":true}},"timer_seconds_duration":0.8,"timer_seconds_left":0.0,"to_block":[4],"blockers":[11]},{"pos":[183.67923,483.67703],"node_type":{"Blocker":{"is_blocked":true}},"timer_seconds_duration":1.0,"timer_seconds_left":0.0,"to_block":[2],"blockers":[]},{"pos":[-274.4962,188.13853],"node_type":{"Blocker":{"is_blocked":true}},"timer_seconds_duration":1.0,"timer_seconds_left":0.0,"to_block":[7],"blockers":[12]},{"pos":[198.78012,23.812317],"node_type":{"Blocker":{"is_blocked":true}},"timer_seconds_duration":1.2,"timer_seconds_left":0.0,"to_block":[8],"blockers":[]},{"pos":[-283.67746,387.93768],"node_type":{"Blocker":{"is_blocked":true}},"timer_seconds_duration":1.2,"timer_seconds_left":0.0,"to_block":[10],"blockers":[13]},{"pos":[-448.68988,500.9638],"node_type":{"Blocker":{"is_blocked":true}},"timer_seconds_duration":1.2,"timer_seconds_left":1.1166811,"to_block":[12],"blockers":[]}]}"#;

    let game_loaded: Save = serde_json::from_str(&j).unwrap();
    println!("{:?}", game_loaded);
    game_loaded
}

pub fn save(
    q_nodes: Query<
        (
            Entity,
            &Transform,
            &Progress,
            Option<&NodeCurrencyGain>,
            &SelfBlockStatus,
            &ToBlock,
            &Blockers,
        ),
        With<BaseNode>,
    >,
) {
    let mut node_entities_index: HashMap<Entity, usize> = HashMap::new();
    for (i, (e, _, _, _, _, _, _)) in q_nodes.iter().enumerate() {
        node_entities_index.insert(e, i);
    }
    let mut nodes = Vec::new();

    for (e, transform, progress, gain, self_status, to_block, blockers) in q_nodes.iter() {
        nodes.push(SavedNode {
            pos: transform.translation.truncate(),
            node_type: match gain {
                Some(g) => NodeType::Gain { level: g.0 },
                None => NodeType::Blocker {
                    is_blocked: self_status.is_blocked,
                },
            },
            timer_seconds_duration: progress.timer.duration().as_secs_f32(),
            timer_seconds_left: progress.timer.remaining_secs(),
            to_block: to_block
                .entities
                .iter()
                .map(|b| node_entities_index[&b])
                .collect(),
            blockers: blockers
                .entities
                .iter()
                .map(|b| node_entities_index[&b])
                .collect(),
        });
    }
    let data = Save {
        last_tick_time_since2023: 2f32,
        nodes,
    };
    let j = serde_json::to_string(&data).unwrap();
    println!("{}", j);
}

#[derive(Reflect, Component)]
pub struct LoadingNode(pub SavedNode);

/// To find LoadingNode indices corresponding entities
#[derive(Reflect, Component)]
pub struct LoadingNodes(pub Vec<Entity>);

pub fn start_load(commands: &mut Commands, save: &Save) {
    let mut node_entities_index: Vec<Entity> = Vec::new();
    for n in &save.nodes {
        node_entities_index.push(commands.spawn(LoadingNode((*n).clone())).id());
    }
    commands.spawn(LoadingNodes(node_entities_index));
}

fn load_system(
    mut commands: Commands,
    map_assets: Res<MapAssets>,
    highlights: Res<HighlightingMaterials>,
    q_loading_nodes: Query<(Entity, &LoadingNodes)>,
    q_individual_node: Query<&LoadingNode>,
) {
    for (e_loading, loading_nodes) in q_loading_nodes.iter() {
        for e_node in loading_nodes.0.iter() {
            let loading_node = q_individual_node.get(*e_node).unwrap();
            let blockers = Blockers {
                entities: loading_node
                    .0
                    .blockers
                    .iter()
                    .map(|index| loading_nodes.0[*index])
                    .collect(),
            };
            let to_block = ToBlock {
                entities: loading_node
                    .0
                    .to_block
                    .iter()
                    .map(|index| loading_nodes.0[*index])
                    .collect(),
            };
            match loading_node.0.node_type {
                NodeType::Blocker { is_blocked } => {
                    insert_node(
                        &mut commands,
                        map_assets.mesh_blocker.clone(),
                        &map_assets,
                        &highlights,
                        loading_node.0.pos,
                        loading_node.0.timer_seconds_duration,
                        *e_node,
                        // TODO: a way to set elapsed time
                    );
                    commands
                        .entity(*e_node)
                        .insert(NodeManualBlockToggle { is_blocked })
                        .insert(SelfBlockStatus { is_blocked: true });

                    commands.entity(*e_node).insert(blockers);
                    commands.entity(*e_node).insert(to_block);
                }
                NodeType::Gain { level } => {
                    insert_node(
                        &mut commands,
                        map_assets.mesh_gain.clone(),
                        &map_assets,
                        &highlights,
                        loading_node.0.pos,
                        loading_node.0.timer_seconds_duration,
                        *e_node,
                    );
                    commands.entity(*e_node).insert(NodeCurrencyGain(level));

                    commands.entity(*e_node).insert(blockers);
                    commands.entity(*e_node).insert(to_block);
                }
            }
            commands.entity(*e_node).remove::<LoadingNode>();
        }
        commands.entity(e_loading).despawn();
    }
}

use bevy::{
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::{
    new_node::{create_node, insert_node, TIMER_BLOCKER_MULT},
    Blockers, MapAssets, NodeCurrencyGain, NodeManualBlockToggle, SelfBlockStatus, ToBlock,
};

pub struct GameLoader;

impl Plugin for GameLoader {
    fn build(&self, app: &mut App) {
        app.register_type::<LoadingNode>();
        app.register_type::<LoadingNodes>();
        app.add_system(load_system);
    }
}

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum NodeType {
    Gain,
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
                node_type: NodeType::Gain,
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

    let game_loaded: Save = serde_json::from_str(&j).unwrap();
    println!("{:?}", game_loaded);
    game_loaded
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
                NodeType::Gain => {
                    insert_node(
                        &mut commands,
                        map_assets.mesh_gain.clone(),
                        &map_assets,
                        loading_node.0.pos,
                        loading_node.0.timer_seconds_duration,
                        *e_node,
                    );
                    commands.entity(*e_node).insert(NodeCurrencyGain);

                    commands.entity(*e_node).insert(blockers);
                    commands.entity(*e_node).insert(to_block);
                }
            }
            commands.entity(*e_node).remove::<LoadingNode>();
        }
        commands.entity(e_loading).despawn();
    }
}

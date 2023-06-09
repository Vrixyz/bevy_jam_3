use bevy::{input::common_conditions::input_just_pressed, prelude::*, utils::HashMap};
use bevy_pkv::PkvStore;
use serde::{Deserialize, Serialize};

use crate::{
    currency,
    idle_gains::Currency,
    new_node::{insert_node, BaseNode, EyeCatcher},
    picking::HighlightingMaterials,
    progress::{NodeTextValidate, Progress},
    Blockers, ButtonRef, MapAssets, NodeCurrencyGain, NodeManualBlockToggle, NodeSave,
    SelfBlockStatus, ToBlock,
};

pub struct GameLoader;

impl Plugin for GameLoader {
    fn build(&self, app: &mut App) {
        app.insert_resource(PkvStore::new("sidleffect", "save"));
        app.register_type::<LoadingNode>();
        app.register_type::<LoadingPending>();
        app.add_event::<EventSave>();
        app.add_system(load_system.in_base_set(CoreSet::PreUpdate));
        app.add_system(save_event);
        app.add_system(save_cheat.run_if(input_just_pressed(KeyCode::S)));

        app.add_system(
            clear
                .in_base_set(CoreSet::PreUpdate)
                .run_if(input_just_pressed(KeyCode::E)),
        );
    }
}

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum NodeType {
    Gain { level: u32 },
    Save { level: u32 },
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
    pub currencies: i32,
    pub last_tick_time_since2023: f32,
    pub nodes: Vec<SavedNode>,
}

pub fn load(pkv: &Res<PkvStore>) -> Result<Save, String> {
    let get_default_save = || Save {
        currencies: 0,
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
    let mut save = pkv
        .get::<Save>("game")
        .unwrap_or_else(|_err| get_default_save());
    if save.nodes.is_empty() {
        save = get_default_save();
    }
    Ok(save)
}

pub struct EventSave;

pub fn save_cheat(mut evt: EventWriter<EventSave>) {
    evt.send(EventSave);
}

pub fn save_event(
    currencies: Res<Currency>,
    q_nodes: Query<
        (
            Entity,
            &Transform,
            &Progress,
            Option<&NodeCurrencyGain>,
            Option<&NodeSave>,
            &SelfBlockStatus,
            &ToBlock,
            &Blockers,
        ),
        With<BaseNode>,
    >,
    mut pkv: ResMut<PkvStore>,
    evt: EventReader<EventSave>,
) {
    if evt.is_empty() {
        return;
    }
    let mut node_entities_index: HashMap<Entity, usize> = HashMap::new();
    for (i, (e, _, _, _, _, _, _, _)) in q_nodes.iter().enumerate() {
        node_entities_index.insert(e, i);
    }
    let mut nodes = Vec::new();

    for (_e, transform, progress, gain, save, self_status, to_block, blockers) in q_nodes.iter() {
        nodes.push(SavedNode {
            pos: transform.translation.truncate(),
            node_type: match (gain, save) {
                (Some(g), _) => NodeType::Gain { level: g.level },
                (_, Some(save)) => NodeType::Save { level: save.level },
                (None, None) => NodeType::Blocker {
                    is_blocked: self_status.is_blocked,
                },
            },
            timer_seconds_duration: progress.timer.duration().as_secs_f32(),
            timer_seconds_left: progress.timer.remaining_secs(),
            to_block: to_block
                .entities
                .iter()
                .map(|b| node_entities_index[b])
                .collect(),
            blockers: blockers
                .entities
                .iter()
                .map(|b| node_entities_index[b])
                .collect(),
        });
    }
    let data = Save {
        currencies: currencies.amount,
        last_tick_time_since2023: 2f32,
        nodes,
    };
    pkv.set("game", &data).expect("pkv should be able to save.");
    let j = serde_json::to_string(&data).unwrap();
    println!("saved: {}", j);
}

#[derive(Reflect, Component)]
pub struct LoadingNode(pub SavedNode);

/// To find LoadingNode indices corresponding entities
#[derive(Reflect, Component)]
pub struct LoadingPending {
    pub currencies: i32,
    pub nodes: Vec<Entity>,
}

pub fn start_load(commands: &mut Commands, save: &Save) {
    let mut node_entities_index: Vec<Entity> = Vec::new();
    for n in &save.nodes {
        node_entities_index.push(commands.spawn(LoadingNode((*n).clone())).id());
    }
    commands.spawn(LoadingPending {
        currencies: save.currencies,
        nodes: node_entities_index,
    });
}

fn clear(
    mut commands: Commands,
    q_old_nodes: Query<(Entity, Option<&EyeCatcher>), Or<(With<ButtonRef>, With<BaseNode>)>>,
) {
    for (e_old, eye) in q_old_nodes.iter() {
        if let Some(eye) = eye {
            commands.entity(eye.0).despawn();
        }
        commands.entity(e_old).despawn();
    }
}

fn load_system(
    mut commands: Commands,
    mut currency: ResMut<Currency>,
    map_assets: Res<MapAssets>,
    highlights: Res<HighlightingMaterials>,
    q_loading_nodes: Query<(Entity, &LoadingPending)>,
    q_individual_node: Query<&LoadingNode>,
    q_old_nodes: Query<(Entity, Option<&EyeCatcher>), Or<(With<ButtonRef>, With<BaseNode>)>>,
) {
    for (e_loading, loading_pending) in q_loading_nodes.iter() {
        for (e_old, eye) in q_old_nodes.iter() {
            if let Some(eye) = eye {
                commands.entity(eye.0).despawn();
            }
            commands.entity(e_old).despawn();
        }
        dbg!("removed all");
        currency.amount = loading_pending.currencies;
        for e_node in loading_pending.nodes.iter() {
            let loading_node = q_individual_node.get(*e_node).unwrap();
            let blockers = Blockers {
                entities: loading_node
                    .0
                    .blockers
                    .iter()
                    .map(|index| loading_pending.nodes[*index])
                    .collect(),
            };
            let to_block = ToBlock {
                entities: loading_node
                    .0
                    .to_block
                    .iter()
                    .map(|index| loading_pending.nodes[*index])
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
                        loading_node.0.timer_seconds_duration - loading_node.0.timer_seconds_left,
                        *e_node,
                    );
                    commands
                        .entity(*e_node)
                        .insert(NodeManualBlockToggle { is_blocked })
                        .insert(SelfBlockStatus { is_blocked: true });

                    commands.entity(*e_node).insert(blockers);
                    commands.entity(*e_node).insert(to_block);
                }
                NodeType::Save { level } => {
                    insert_node(
                        &mut commands,
                        map_assets.mesh_save.clone(),
                        &map_assets,
                        &highlights,
                        loading_node.0.pos,
                        loading_node.0.timer_seconds_duration,
                        loading_node.0.timer_seconds_duration - loading_node.0.timer_seconds_left,
                        *e_node,
                    );
                    commands.entity(*e_node).insert(NodeTextValidate {
                        text: "Save".to_string(),
                    });
                    commands.entity(*e_node).insert(NodeSave { level });
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
                        loading_node.0.timer_seconds_duration - loading_node.0.timer_seconds_left,
                        *e_node,
                    );
                    commands.entity(*e_node).insert(NodeCurrencyGain { level });

                    commands.entity(*e_node).insert(NodeTextValidate {
                        text: "Gain!".to_string(),
                    });
                    commands.entity(*e_node).insert(blockers);
                    commands.entity(*e_node).insert(to_block);
                }
            }
            commands.entity(*e_node).remove::<LoadingNode>();
        }
        dbg!("loadedall");
        commands.entity(e_loading).despawn();
    }
}

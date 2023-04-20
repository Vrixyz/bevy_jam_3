// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::time::Duration;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy::input::common_conditions::input_just_pressed;
use bevy::{input::common_conditions::input_toggle_active, prelude::*, sprite::Mesh2dHandle};
use bevy_easings::EasingsPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::events::PointerEvent;
use bevy_mod_picking::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_pkv::PkvStore;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use idle_gains::Currency;
use new_node::*;
use persisted_game::GameLoader;
use poisson::Poisson;
use progress::Progress;
use rand::{Rng, SeedableRng};

mod idle_gains;
mod new_node;
pub mod persisted_game;
use picking::auto_click;
use picking::HighlightingMaterials;
use picking_aabb::AabbBackend;
use progress::*;

use status_visual::update_status_visual;

//pub mod persisted_game;
pub mod picking;
mod picking_aabb;
mod poisson;
mod progress;
mod status_visual;
pub mod timer_material;

use timer_material::{TimerMaterial, TimerMaterialPlugin, TimerMaterials};

fn main() {
    App::new()
        .register_type::<Blockers>()
        .register_type::<ToBlock>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                //resolution: WindowResolution::new(640., 640.).with_scale_factor_override(1.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(DefaultPickingPlugins) // <- Adds picking, interaction, and highlighting
        .add_plugin(GameLoader)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(AabbBackend)
        //.add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging.
        .add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(PanCamPlugin::default())
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(EasingsPlugin)
        .add_plugin(TimerMaterialPlugin)
        .add_event::<NewNodeEvent>()
        .add_event::<PropagateResetManualButtons>()
        .init_resource::<Currency>()
        .add_startup_system(picking::setup)
        .add_system(
            setup
                .in_schedule(CoreSchedule::Startup)
                .in_base_set(StartupSet::PostStartup),
        )
        .add_system(
            load.in_schedule(CoreSchedule::Startup)
                .in_base_set(StartupSet::PostStartup),
        )
        .add_system(load.run_if(input_just_pressed(KeyCode::L)))
        /*        .add_system(
            spawn_map_empty
                .in_schedule(CoreSchedule::Startup)
                .in_base_set(StartupSet::PostStartup),
        )*/
        .add_system(update_progress_timer)
        .add_system(update_progress_manual_auto_block)
        .add_system(picking::button_react)
        .add_system(reset_manual_button_timers.after(picking::button_react))
        .add_system(button_manual_toggle_block_react)
        .add_system(new_button)
        .add_system(
            check_self_block
                .after(new_button)
                .after(reset_manual_button_timers),
        )
        .add_system(update_inherited_block_status.after(check_self_block))
        .add_system(update_progress_text.after(update_inherited_block_status))
        .add_system(update_progress_material.after(update_inherited_block_status))
        .add_system(draw_relations.before(new_button))
        .add_system(update_status_visual.after(update_inherited_block_status))
        .add_system(
            auto_click
                .after(update_status_visual)
                .run_if(input_toggle_active(false, KeyCode::C)),
        )
        .run();
}

fn load(mut commands: Commands, pkv: Res<PkvStore>) {
    let Ok(to_load) = persisted_game::load(&pkv) else {
        return;
    };
    persisted_game::start_load(&mut commands, &to_load);
}

#[derive(Component)]
pub struct ButtonRef(pub Entity);

// node behaviours

#[derive(Component)]
pub struct NodeManualBlockToggle {
    pub is_blocked: bool,
}
#[derive(Component)]
pub struct NodeCurrencyGain(pub u32);
//

/// To know which nodes are blocking our behaviour.
#[derive(Reflect, Component)]
pub struct Blockers {
    pub entities: Vec<Entity>,
}

/// To know which nodes to block.
#[derive(Reflect, Component)]
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
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets_timer: ResMut<Assets<TimerMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(RandomForMap::default());
    // 2d camera
    commands.spawn((
        Camera2dBundle::default(),
        PanCam::default(),
        PickRaycastCamera::default(),
    ));
    dbg!("setup main");
    commands.spawn(TimerMaterials::new(&mut assets_timer, Color::GREEN, 180));
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
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
    };

    commands.insert_resource(map_assets);
}

fn button_manual_toggle_block_react(
    mut events: EventReader<PointerEvent<Down>>,
    mut q_nodes: Query<(&Progress, &mut NodeManualBlockToggle, &InheritedBlockStatus)>,
) {
    for event in events.iter() {
        let e = event.target();
        let Ok((p, mut node, status)) = q_nodes.get_mut(e) else {
            continue;
        };
        if status.is_blocked {
            //dbg!("node is blocked");
            continue;
        }
        if p.timer.finished() {
            //dbg!("toggle block!");
            node.is_blocked = !node.is_blocked;
        } else {
            //dbg!("NOT READY");
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
    mut lines: ResMut<DebugLines>,
    q_to_block: Query<(&ToBlock, Entity), With<BaseNode>>,
    q_blockers: Query<
        (
            &Transform,
            &Blockers,
            Entity,
            &InheritedBlockStatus,
            &SelfBlockStatus,
        ),
        With<BaseNode>,
    >,
) {
    for (to_blocks, e) in q_to_block.iter() {
        if to_blocks.entities.is_empty() {
            recurse_draw_relations(&mut lines, &q_blockers, e);
        }
    }
}

fn recurse_draw_relations(
    lines: &mut ResMut<DebugLines>,
    q_blockers: &Query<
        (
            &Transform,
            &Blockers,
            Entity,
            &InheritedBlockStatus,
            &SelfBlockStatus,
        ),
        With<BaseNode>,
    >,
    e: Entity,
) {
    if let Ok(blocked) = q_blockers.get(e) {
        for new_blocker in blocked.1.entities.iter() {
            let blocker_data = q_blockers.get(*new_blocker).unwrap();
            lines.line_colored(
                blocked.0.translation,
                q_blockers
                    .get(*new_blocker)
                    .expect("all nodes have a transform")
                    .0
                    .translation,
                0f32,
                if blocker_data.3.is_blocked || blocker_data.4.is_blocked {
                    Color::RED
                } else {
                    Color::GREEN
                },
            );
            recurse_draw_relations(lines, q_blockers, *new_blocker);
        }
    }
}

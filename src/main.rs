// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_picking::{
    DebugEventsPickingPlugin, DefaultPickingPlugins, PickableBundle, PickingCameraBundle,
    PickingEvent,
};
use progress::Progress;

mod idle_gains;
mod progress;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins) // <- Adds picking, interaction, and highlighting
        .add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging.
        .add_startup_system(setup)
        .add_system(update_progress_text)
        .add_system(update_progress_timer)
        .add_system(button_react)
        .run();
}

#[derive(Component)]
pub struct ButtonRef(pub Entity);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 30.0,
        color: Color::WHITE,
    };
    let text_alignment = TextAlignment::Center;
    // 2d camera
    commands.spawn((Camera2dBundle::default(), PickingCameraBundle::default()));
    let button_entity = commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                transform: Transform::default().with_scale(Vec3::splat(128.)),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                ..default()
            },
            PickableBundle::default(),
            Progress {
                timer: Timer::from_seconds(1f32, TimerMode::Once),
            },
        ))
        .id();
    // Demonstrate changing translation
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("translation", text_style.clone())
                .with_alignment(text_alignment),
            ..default()
        },
        ButtonRef(button_entity),
    ));
}

fn update_progress_timer(time: Res<Time>, mut q_timer: Query<&mut Progress>) {
    for mut t in q_timer.iter_mut() {
        if !t.timer.finished() {
            t.timer.tick(time.delta());
        }
    }
}
fn update_progress_text(
    mut q_texts: Query<(&mut Text, &ButtonRef)>,
    q_timer: Query<&Progress, Changed<Progress>>,
) {
    for (mut t, b) in q_texts.iter_mut() {
        match q_timer.get(b.0) {
            Err(_) => {}
            Ok(p) => {
                if p.timer.finished() {
                    t.sections[0].value = format!("Gain!");
                } else {
                    t.sections[0].value = format!("{}s", p.timer.remaining_secs());
                }
            }
        }
    }
}

pub fn button_react(mut events: EventReader<PickingEvent>, mut q_timer: Query<&mut Progress>) {
    for event in events.iter() {
        if let PickingEvent::Clicked(e) = event {
            let mut p = q_timer.get_mut(*e).unwrap();
            if p.timer.finished() {
                dbg!("GAIN!");
                p.timer.reset();
            } else {
                dbg!("NOT READY");
            }
        }
    }
}

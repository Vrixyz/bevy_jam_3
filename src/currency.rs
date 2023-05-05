use std::string;

use bevy::{prelude::*, transform};
use bevy_easings::Ease;

use crate::idle_gains::Currency;

pub struct CurrencyPlugin;

impl Plugin for CurrencyPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(currency_change_react);
    }
}
// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText;

// A unit struct to help identify the color-changing Text component
#[derive(Component)]
struct CurrencyText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        font_size: 100.0,
        color: Color::WHITE,
    };
    commands.spawn((
        TextBundle::from_sections(vec![
            TextSection::new("coins:", style.clone()),
            TextSection::new("0", style.clone()),
        ])
        // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                right: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
        CurrencyText,
    ));
}

fn currency_change_react(
    mut commands: Commands,
    currency: Res<Currency>,
    mut query: Query<(Entity, &Transform, &mut Text), With<CurrencyText>>,
) {
    if currency.is_changed() {
        for (e, transform, mut text) in query.iter_mut() {
            text.sections[1].value = currency.amount.to_string();
            commands.entity(e).insert(
                transform
                    .ease_to(
                        Transform {
                            scale: Vec3::new(
                                transform.scale.x * 1.5f32,
                                transform.scale.y * 1.5f32,
                                transform.scale.z * 1.5f32,
                            ),
                            ..transform.clone()
                        },
                        bevy_easings::EaseFunction::QuadraticInOut,
                        bevy_easings::EasingType::Once {
                            duration: std::time::Duration::from_millis(200),
                        },
                    )
                    .ease_to(
                        Transform {
                            scale: Vec3::new(1f32, 1f32, 1f32),
                            ..transform.clone()
                        },
                        bevy_easings::EaseFunction::QuadraticInOut,
                        bevy_easings::EasingType::Once {
                            duration: std::time::Duration::from_millis(200),
                        },
                    ),
            );
        }
    }
}

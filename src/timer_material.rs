//! A shader and a material that uses it.

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};

pub struct TimerMaterialPlugin;

impl Plugin for TimerMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<TimerMaterial>::default());
    }
}

#[derive(Component)]
pub struct TimerMaterials {
    /// Contains materials from progress 0 to progress 1
    pub materials: Vec<Handle<TimerMaterial>>,
}

impl TimerMaterials {
    pub fn new(
        assets: &mut ResMut<Assets<TimerMaterial>>,
        color: Color,
        // Amount of materials created, should be > 0
        resolution: usize,
    ) -> TimerMaterials {
        debug_assert!(resolution > 0);
        let mut materials = Vec::<Handle<TimerMaterial>>::with_capacity(resolution);
        for i in 0..=resolution {
            materials.push(assets.add(TimerMaterial {
                color,
                progress: dbg!(i as f32 / (resolution as f32)),
            }))
        }
        TimerMaterials { materials }
    }
    pub fn get_material(&self, progress: f32) -> Handle<TimerMaterial> {
        self.materials[(progress * (self.materials.len() as f32)) as usize].clone()
    }
    // TODO: release materials when done with it.
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material2d for TimerMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/timer_material.wgsl".into()
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct TimerMaterial {
    #[uniform(0)]
    color: Color,
    /// From 0 to 1
    #[uniform(0)]
    progress: f32,
}

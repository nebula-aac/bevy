use crate::{
    extract_resource::ExtractResource,
    load_shader_library,
    render_resource::{ShaderType, UniformBuffer},
    renderer::{RenderDevice, RenderQueue},
    Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
};
use bevy_app::{App, Plugin};
use bevy_diagnostic::FrameCount;
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use bevy_time::Time;

pub struct GlobalsPlugin;

impl Plugin for GlobalsPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "globals.wgsl");
        app.register_type::<GlobalsUniform>();

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<GlobalsBuffer>()
                .init_resource::<Time>()
                .add_systems(ExtractSchedule, (extract_frame_count, extract_time))
                .add_systems(
                    Render,
                    prepare_globals_buffer.in_set(RenderSystems::PrepareResources),
                );
        }
    }
}

fn extract_frame_count(mut commands: Commands, frame_count: Extract<Res<FrameCount>>) {
    commands.insert_resource(**frame_count);
}

fn extract_time(mut commands: Commands, time: Extract<Res<Time>>) {
    commands.insert_resource(**time);
}

/// Contains global values useful when writing shaders.
/// Currently only contains values related to time.
#[derive(Default, Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource, Default, Clone)]
pub struct GlobalsUniform {
    /// The time since startup in seconds.
    /// Wraps to 0 after 1 hour.
    time: f32,
    /// The delta time since the previous frame in seconds
    delta_time: f32,
    /// Frame count since the start of the app.
    /// It wraps to zero when it reaches the maximum value of a u32.
    frame_count: u32,
    /// WebGL2 structs must be 16 byte aligned.
    #[cfg(all(feature = "webgl", target_arch = "wasm32", not(feature = "webgpu")))]
    _wasm_padding: f32,
}

/// The buffer containing the [`GlobalsUniform`]
#[derive(Resource, Default)]
pub struct GlobalsBuffer {
    pub buffer: UniformBuffer<GlobalsUniform>,
}

fn prepare_globals_buffer(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut globals_buffer: ResMut<GlobalsBuffer>,
    time: Res<Time>,
    frame_count: Res<FrameCount>,
) {
    let buffer = globals_buffer.buffer.get_mut();
    buffer.time = time.elapsed_secs_wrapped();
    buffer.delta_time = time.delta_secs();
    buffer.frame_count = frame_count.0;

    globals_buffer
        .buffer
        .write_buffer(&render_device, &render_queue);
}

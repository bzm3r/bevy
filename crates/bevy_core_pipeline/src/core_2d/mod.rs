mod camera_2d;
pub mod graph;
mod main_pass_2d_node;

pub use camera_2d::*;
pub use main_pass_2d_node::*;

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_render::{
    camera::Camera,
    extract_component::ExtractComponentPlugin,
    render_phase::{
        batch_phase_system, sort_phase_system, BatchedPhaseItem, CachedRenderPipelinePhaseItem,
        DrawFunctionId, DrawFunctions, PhaseItem, RenderPhase,
    },
    render_resource::CachedRenderPipelineId,
    Extract, ExtractSchedule, Render, RenderApp, RenderSet,
};
use bevy_utils::FloatOrd;
use std::ops::Range;

pub use self::graph::*;

#[derive(Clone, Debug, Default)]
pub struct Core2dPlugin {
    pub core_pipeline_settings: Core2dSettings,
}

impl Plugin for Core2dPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Camera2d>()
            .add_plugins(ExtractComponentPlugin::<Camera2d>::default());

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app
            .init_resource::<DrawFunctions<Transparent2d>>()
            .add_systems(ExtractSchedule, extract_core_2d_camera_phases)
            .add_systems(
                Render,
                (
                    sort_phase_system::<Transparent2d>.in_set(RenderSet::PhaseSort),
                    batch_phase_system::<Transparent2d>
                        .after(sort_phase_system::<Transparent2d>)
                        .in_set(RenderSet::PhaseSort),
                ),
            );

        use graph::*;
        let core_sequence = create_core_sequence(self.core_pipeline_settings);
        core_sequence.create_new_sub_graph(render_app, CORE_2D);
    }
}

pub struct Transparent2d {
    pub sort_key: FloatOrd,
    pub entity: Entity,
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    /// Range in the vertex buffer of this item
    pub batch_range: Option<Range<u32>>,
}

impl PhaseItem for Transparent2d {
    type SortKey = FloatOrd;

    #[inline]
    fn entity(&self) -> Entity {
        self.entity
    }

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    #[inline]
    fn sort(items: &mut [Self]) {
        items.sort_by_key(|item| item.sort_key());
    }
}

impl CachedRenderPipelinePhaseItem for Transparent2d {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}

impl BatchedPhaseItem for Transparent2d {
    fn batch_range(&self) -> &Option<Range<u32>> {
        &self.batch_range
    }

    fn batch_range_mut(&mut self) -> &mut Option<Range<u32>> {
        &mut self.batch_range
    }
}

pub fn extract_core_2d_camera_phases(
    mut commands: Commands,
    cameras_2d: Extract<Query<(Entity, &Camera), With<Camera2d>>>,
) {
    for (entity, camera) in &cameras_2d {
        if camera.is_active {
            commands
                .get_or_spawn(entity)
                .insert(RenderPhase::<Transparent2d>::default());
        }
    }
}

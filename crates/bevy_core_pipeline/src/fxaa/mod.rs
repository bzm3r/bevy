use crate::{
    core_2d::{self, CORE_2D},
    core_3d::{self, CORE_3D},
    fullscreen_vertex_shader::fullscreen_shader_vertex_state,
};
use bevy_app::prelude::*;
use bevy_asset::{load_internal_asset, HandleUntyped};
use bevy_derive::Deref;
use bevy_ecs::prelude::*;
use bevy_reflect::{
    std_traits::ReflectDefault, FromReflect, Reflect, ReflectFromReflect, TypeUuid,
};
use bevy_render::{
    extract_component::{ExtractComponent, ExtractComponentPlugin},
    prelude::Camera,
    render_graph::RenderGraphApp,
    render_graph::ViewNodeRunner,
    render_resource::*,
    renderer::RenderDevice,
    texture::BevyDefault,
    view::{ExtractedView, ViewTarget},
    Render, RenderApp, RenderSet,
};

mod node;

pub use node::FxaaNode;

#[derive(Reflect, FromReflect, Eq, PartialEq, Hash, Clone, Copy)]
#[reflect(FromReflect, PartialEq, Hash)]
pub enum Sensitivity {
    Low,
    Medium,
    High,
    Ultra,
    Extreme,
}

impl Sensitivity {
    pub fn get_str(&self) -> &str {
        match self {
            Sensitivity::Low => "LOW",
            Sensitivity::Medium => "MEDIUM",
            Sensitivity::High => "HIGH",
            Sensitivity::Ultra => "ULTRA",
            Sensitivity::Extreme => "EXTREME",
        }
    }
}

#[derive(Reflect, FromReflect, Component, Clone, ExtractComponent)]
#[reflect(Component, FromReflect, Default)]
#[extract_component_filter(With<Camera>)]
pub struct Fxaa {
    /// Enable render passes for FXAA.
    pub enabled: bool,

    /// Use lower sensitivity for a sharper, faster, result.
    /// Use higher sensitivity for a slower, smoother, result.
    /// [Ultra](`Sensitivity::Ultra`) and [Extreme](`Sensitivity::Extreme`)
    /// settings can result in significant smearing and loss of detail.

    /// The minimum amount of local contrast required to apply algorithm.
    pub edge_threshold: Sensitivity,

    /// Trims the algorithm from processing darks.
    pub edge_threshold_min: Sensitivity,
}

impl Default for Fxaa {
    fn default() -> Self {
        Fxaa {
            enabled: true,
            edge_threshold: Sensitivity::High,
            edge_threshold_min: Sensitivity::High,
        }
    }
}

const FXAA_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4182761465141723543);

/// Adds support for Fast Approximate Anti-Aliasing (FXAA)
#[derive(Default, Clone, Copy)]
pub struct FxaaPlugin {
    /// Specifies whether tonemapping was enabled for the core2d pipeline
    tonemapping2d: bool,
    /// Specifies whether tonemapping was enabled for the core3d pipeline,
    tonemapping3d: bool,
}

impl Plugin for FxaaPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, FXAA_SHADER_HANDLE, "fxaa.wgsl", Shader::from_wgsl);

        app.add_plugin(ExtractComponentPlugin::<Fxaa>::default());

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };
        render_app
            .init_resource::<SpecializedRenderPipelines<FxaaPipeline>>()
            .add_systems(Render, prepare_fxaa_pipelines.in_set(RenderSet::Prepare))
            .add_render_graph_node::<ViewNodeRunner<FxaaNode>>(CORE_3D, core_3d::graph::node::FXAA)
            .add_render_graph_edges(
                CORE_3D,
                &self.generate_3d_edges(),
            )
            .add_render_graph_node::<ViewNodeRunner<FxaaNode>>(CORE_2D, core_2d::graph::node::FXAA)
            .add_render_graph_edges(
                CORE_2D,
                &self.generate_2d_edges(),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };
        render_app.init_resource::<FxaaPipeline>();
    }
}

impl FxaaPlugin {
    /// Generate required edges specifying where the plugin is inserted in the core 2d pipeline
    /// based on user provided settings.
    ///
    /// If tonemapping is enabled for the core 2d pipeline, the edges will be:
    ///     `[END_MAIN_PASS, BLOOM, TONEMAPPING]`
    /// Otherwise, the edges will be:
    ///     `[MAIN_PASS, BLOOM, END_MAIN_PASS_POST_PROCESSING]`
    fn generate_2d_edges(&self) -> [&'static str; 3] {
        let following_node = if self.tonemapping2d {
            core_2d::graph::node::TONEMAPPING
        } else {
            core_2d::graph::node::END_MAIN_PASS_POST_PROCESSING
        };

        [
            core_2d::graph::node::MAIN_PASS,
            core_2d::graph::node::BLOOM,
            following_node,
        ]
    }

    /// Generate required edges specifying where the plugin is inserted in the core 3d pipeline
    /// based on user provided settings.
    ///
    /// If tonemapping is enabled for the core 3d pipeline, the edges will be:
    ///     `[END_MAIN_PASS, FXAA, END_MAIN_PASS_POST_PROCESSING]`
    /// Otherwise, the edges will be:
    ///     `[TONEMAPPING, FXAA, END_MAIN_PASS_POST_PROCESSING]`
    fn generate_3d_edges(&self) -> [&'static str; 3] {
        let following_node = if self.tonemapping2d {
            core_3d::graph::node::TONEMAPPING
        } else {
            core_3d::graph::node::END_MAIN_PASS
        };

        [
            core_3d::graph::node::TONEMAPPING,
            core_3d::graph::node::FXAA,
            core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
        ]
    }
}

#[derive(Resource, Deref)]
pub struct FxaaPipeline {
    texture_bind_group: BindGroupLayout,
}

impl FromWorld for FxaaPipeline {
    fn from_world(render_world: &mut World) -> Self {
        let texture_bind_group = render_world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("fxaa_texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        FxaaPipeline { texture_bind_group }
    }
}

#[derive(Component)]
pub struct CameraFxaaPipeline {
    pub pipeline_id: CachedRenderPipelineId,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct FxaaPipelineKey {
    edge_threshold: Sensitivity,
    edge_threshold_min: Sensitivity,
    texture_format: TextureFormat,
}

impl SpecializedRenderPipeline for FxaaPipeline {
    type Key = FxaaPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("fxaa".into()),
            layout: vec![self.texture_bind_group.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: FXAA_SHADER_HANDLE.typed(),
                shader_defs: vec![
                    format!("EDGE_THRESH_{}", key.edge_threshold.get_str()).into(),
                    format!("EDGE_THRESH_MIN_{}", key.edge_threshold_min.get_str()).into(),
                ],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: key.texture_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: Vec::new(),
        }
    }
}

pub fn prepare_fxaa_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<FxaaPipeline>>,
    fxaa_pipeline: Res<FxaaPipeline>,
    views: Query<(Entity, &ExtractedView, &Fxaa)>,
) {
    for (entity, view, fxaa) in &views {
        if !fxaa.enabled {
            continue;
        }
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &fxaa_pipeline,
            FxaaPipelineKey {
                edge_threshold: fxaa.edge_threshold,
                edge_threshold_min: fxaa.edge_threshold_min,
                texture_format: if view.hdr {
                    ViewTarget::TEXTURE_FORMAT_HDR
                } else {
                    TextureFormat::bevy_default()
                },
            },
        );

        commands
            .entity(entity)
            .insert(CameraFxaaPipeline { pipeline_id });
    }
}

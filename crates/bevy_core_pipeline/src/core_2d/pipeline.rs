use crate::msaa_writeback::MsaaWritebackNode;
use crate::pipelining::PipelineNode;
use crate::tonemapping::TonemappingNode;
use crate::{pipelining::PipelineSequence, tonemapping};
use bevy_render::render_graph::{EmptyNode, ViewNodeRunner};
use paste::paste;

use super::MainPass2dNode;
use crate::bloom::BloomNode;
use crate::pipeline_node;
use crate::upscaling::UpscalingNode;

/// Name of the subgraph of the core 2d pipeline.
pub const CORE_2D: &str = "core_2d";

pub mod input {
    pub const VIEW_ENTITY: &str = "view_entity";
}

pipeline_node!(MsaaWriteback, MsaaWritebackNode);
pipeline_node!(MainPass, MainPass2dNode);
pipeline_node!(Bloom, ViewNodeRunner<BloomNode>);
pipeline_node!(Tonemapping, ViewNodeRunner<TonemappingNode>);
pipeline_node!(Upscaling, ViewNodeRunner<UpscalingNode>);
pipeline_node!(EndMainPassPostProcessing, EmptyNode);

/// Controls how [`create_core_pipeline_sequence`] generates the core 2d rendering pipeline.
#[derive(Clone, Debug)]
pub struct Core2dPipelineSettings {
    /// The label given to this pipeline in the render graph.
    ///
    /// By default, this is `"core_2d"`.
    pub pipeline_label: &'static str,
    /// Controls whether the tonemapping node is enabled.
    ///
    /// By default, this is `true`.
    pub tonemapping: bool,
    tonemapping_label: String,
    /// Controls whether the bloom node is enabled.
    ///
    /// By default, this is `true`.
    pub bloom: bool,
    bloom_label: String,
}

impl Default for Core2dPipelineSettings {
    fn default() -> Self {
        Core2dPipelineSettings {
            pipeline_label: "core_2d",
            tonemapping: true,
            tonemapping_label: Tonemapping::default().label().into(),
            bloom: true,
            bloom_label: Bloom::default().label().into(),
        }
    }
}

macro_rules! check_equality_and_return {
    (node_label: ident, node_name: ident) => {
        paste! {
            if &self.[<$node_name _label>] == $node_label {
                return self.$node_name
            }
        }
    };
}

impl Core2dPipelineSettings {
    pub fn test_inclusion(&self, node: &impl PipelineNode) -> bool {
        let node_label = node.label();

        match node_label {
            _ if &self.tonemapping_label == node_label => self.tonemapping,
            _ if &self.bloom_label == node_label => self.bloom,
            _ => true,
        }
    }
}

/// Creates the default Core 2D rendering pipeline. It consists of the following nodes in sequence:
/// [`MainPass`], [Tonemapping], [`EndMainPassPostProcessing`], [`Upscaling`]
pub fn create_core_pipeline_sequence(settings: Core2dPipelineSettings) -> PipelineSequence {
    let default_sequence = vec![
        MainPass::default(),
        Bloom::default(),
        Tonemapping::default(),
        EndMainPassPostProcessing::default(),
        Upscaling::default(),
    ];

    let tonemapping_label = Tonemapping::default().to_string();

    PipelineSequence::new(
        settings.pipeline_label,
        default_sequence
            .into_iter()
            .filter(|x| settings.test_inclusion(x))
            .collect(),
    )
}

// pipeline_node!(Fxaa, FxaaNode);
// pipeline_node!(Bloom, BloomNode);

use bevy_render::render_graph::{EmptyNode, ViewNodeRunner};
use paste::paste;

use super::MainPass2dNode;
use crate::bloom::BloomNode;
use crate::msaa_writeback::MsaaWritebackNode;
use crate::pipeline_node;
use crate::pipelining::{DynamicPipelineNode, PipelineSequence};
use crate::tonemapping::TonemappingNode;
use crate::upscaling::UpscalingNode;

/// Name of the subgraph of the core 2d pipeline.
pub const CORE_2D: &str = "core_2d";

pub mod input {
    pub const VIEW_ENTITY: &str = "view_entity";
}

pipeline_node!(MainPass, MainPass2dNode);
pipeline_node!(Bloom, ViewNodeRunner<BloomNode>);
pipeline_node!(Tonemapping, ViewNodeRunner<TonemappingNode>);
pipeline_node!(Upscaling, ViewNodeRunner<UpscalingNode>);
pipeline_node!(EndMainPassPostProcessing, EmptyNode);
pipeline_node!(MsaaWriteback, MsaaWritebackNode);

/// Controls how [`create_core_pipeline_sequence`] generates the core 2d rendering pipeline.
#[derive(Clone, Copy, Debug)]
pub struct Core2dPipelineSettings {
    /// Controls whether the tonemapping node is enabled.
    ///
    /// By default, this is `true`.
    pub tonemapping: bool,
    tonemapping_label: &'static str,
    /// Controls whether the bloom node is enabled.
    ///
    /// By default, this is `true`.
    pub bloom: bool,
    bloom_label: &'static str,
    /// Controls whether the MSAA writeback pipeline is attached to the core 2d pipeline.
    ///
    /// By default, this is `true`.
    pub msaa_writeback: bool,
    msaa_writeback_label: &'static str,
}

impl Default for Core2dPipelineSettings {
    fn default() -> Self {
        Core2dPipelineSettings {
            tonemapping: true,
            tonemapping_label: TONEMAPPING,
            bloom: true,
            bloom_label: BLOOM,
            msaa_writeback: true,
            msaa_writeback_label: MSAA_WRITEBACK,
        }
    }
}

macro_rules! check_equality_and_return {
    ($self:ident, $node_label: ident, $node_name: ident) => {
        paste! {
            if $self.[<$node_name _label>] == $node_label {
                return $self.$node_name;
            }
        }
    };
}

impl Core2dPipelineSettings {
    pub fn test_inclusion(&self, node: &DynamicPipelineNode) -> bool {
        let node_label = node.label();

        check_equality_and_return!(self, node_label, tonemapping);
        check_equality_and_return!(self, node_label, bloom);
        check_equality_and_return!(self, node_label, msaa_writeback);

        true
    }
}

/// Creates the default Core 2D rendering pipeline. It consists of the following nodes in sequence:
/// [`MainPass`], [Tonemapping], [`EndMainPassPostProcessing`], [`Upscaling`]
pub fn create_core_sequence(settings: Core2dPipelineSettings) -> PipelineSequence {
    let default_sequence: Vec<DynamicPipelineNode> = vec![
        MainPass::new(),
        Bloom::new(),
        Tonemapping::new(),
        EndMainPassPostProcessing::new(),
        Upscaling::new(),
    ];

    PipelineSequence::new(
        CORE_2D,
        default_sequence
            .into_iter()
            .filter(|x| settings.test_inclusion(x))
            .collect(),
    )
}

/// Creates the default Core 2D rendering pipeline. It consists of the following nodes in sequence:
/// [`MainPass`], [Tonemapping], [`EndMainPassPostProcessing`], [`Upscaling`]
pub fn create_msaa_writeback_sequence(settings: Core2dPipelineSettings) -> PipelineSequence {
    let default_sequence: Vec<DynamicPipelineNode> = vec![MsaaWriteback::new()];

    PipelineSequence::new(
        MSAA_WRITEBACK,
        default_sequence
            .into_iter()
            .filter(|x| settings.test_inclusion(x))
            .collect(),
    )
}

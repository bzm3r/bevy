use std::hash::Hash;

use bevy_render::render_graph::{EmptyNode, ViewNodeRunner};
use bevy_utils::hashbrown::HashMap;
use paste::paste;

use super::MainPass2dNode;
use crate::bloom::BloomNode;
use crate::fxaa::FxaaNode;
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

/// Pipeline nodes that are absolutely required for the core 2d pipeline to do its work.
pub mod required {
    use super::*;
    pipeline_node!(MainPass, MainPass2dNode);
    pipeline_node!(Upscaling, ViewNodeRunner<UpscalingNode>);
    pipeline_node!(EndMainPassPostProcessing, EmptyNode);
}

/// Optional nodes that are inserted into the core sequence based on [`Core2dPipelineSettings`].
/// If all of [`Bloom`](optional::Bloom), [`Tonemapping`](optional::Tonemapping) and [`Fxaa`] are
/// enabled, then the following pipeline sequence would be generated:
/// ```
/// let core_2d_sequence = vec![
///        required::MainPass::new(),
///        optional::Bloom::new(),
///        optional::Tonemapping::new(),
///        optional::Fxaa::new(),
///        required::EndMainPassPostProcessing::new(),
///        required::Upscaling::new(),
///    ];
/// ```
///
/// Otherwise, a sequence in the same order as above would be generated,
/// but with some of the optional nodes removed. If [`MsaaWriteback`](optional::MSAA_WRITEBACK) is enabled,
/// then the following additional sequence is inserted into the [`CORE_2D`] graph, with the existing [`MainPass`]
/// as its target.
pub mod optional {
    use super::*;
    pipeline_node!(Bloom, ViewNodeRunner<BloomNode>);
    pipeline_node!(Tonemapping, ViewNodeRunner<TonemappingNode>);
    pipeline_node!(Fxaa, FxaaNode);
    pipeline_node!(MsaaWriteback, MsaaWritebackNode);
}

use optional::*;
use required::*;

#[derive(Clone, Debug)]
pub struct Core2dPipelineSettings(HashMap<&'static str, bool>);

impl Default for Core2dPipelineSettings {
    fn default() -> Self {
        Core2dPipelineSettings(HashMap::from([
            (TONEMAPPING, true),
            (BLOOM, true),
            (MSAA_WRITEBACK, true),
        ]))
    }
}

#[macro_export]
macro_rules! check_equality_and_return {
    ($self:ident, $node_label: ident, $node_id: ident) => {
        paste! {
            if $self.[<$node_id _label>] == $node_label {
                return $self.$node_id;
            }
        }
    };
}

#[macro_export]
macro_rules! test_sequence_inclusion {
    ( $sequence_name:ident, $($node_id:ident),* ) => {
        paste! {
            #[doc = "Test whether a [`DynamicPipelineNode`](bevy_core_pipeline::pipelining::DynamicPipelineNode) 
            should be included in the final sequence generated for a " $sequence_name ", based on this settings structure. 
            
            A node will be tested for inclusion if its [`label`](bevy_core_pipeline::pipelining::PipelineNode::label) matches 
            one of the labels pre-defined for this function. These pre-defined labels are: "]
            $(#[doc = "* [`" $node_id:upper "`]" ])*
            #[doc = "If no such match exists, this function returns `true`.Otherwise, this function returns 
            the `bool` associated with that label."]
            pub fn [<test_ $sequence_name _sequence_inclusion>](&self, node: &DynamicPipelineNode) -> bool {
                for test_label in [$([<$node_id:upper>]),*].into_iter() {
                    if test_label == node.label() {
                        if let Some(result) = self.0.get(test_label) {
                            return result;
                        }
                    }
                }
                // return true by default
                true
            }
        }
    }
}

impl Core2dPipelineSettings {
    test_sequence_inclusion!(core, tonemapping, bloom);

    test_sequence_inclusion!(msaa_writeback, msaa_writeback);
}

macro_rules! create_simple_sequencer {
    ( $sequence_description:literal, $sequence_id:ident; $($node_ty:ty),+ $(; $settings_type:ty)? ) => {
        paste! {
            #[doc = "Creates the " $sequence_description " [`PipelineSequence`]. It consists of the following nodes 
            in sequence" $(", but some might be enabled/disabled based on [`" $settings_type "`]'s 
            configuration (see the [`required`] and [`optional`] sub-modules. for further explanation)")? ":\n"]
            #[doc = "" $("[`" $node_ty "`]")" `->` "+ "" ]
            pub fn [<create_ $sequence_id _sequence>]($([< $settings_type:lower:snake  >]: $settings_type)?) -> PipelineSequence {
                use optional::*;
                use required::*;

                let node_sequence: Vec<DynamicPipelineNode> = vec![$($node_ty::new()),+];

                PipelineSequence::new(
                    CORE_2D,
                    node_sequence
                        .into_iter()
                        $(.filter(|x| [<$settings_type:lower:snake>].[<test_ $sequence_id _sequence_inclusion>](x)))?
                        .collect(),
                )
            }
        }
    }
}

create_simple_sequencer!(
    "core 2d",
    core;
    MainPass,
    Bloom,
    Tonemapping,
    Fxaa,
    EndMainPassPostProcessing,
    Upscaling;
    Core2dPipelineSettings
);

// pub fn create_core_sequence(settings: Core2dPipelineSettings) -> PipelineSequence {
//     use optional::*;
//     use required::*;

//     let node_sequence: Vec<DynamicPipelineNode> = vec![
//         MainPass::new(),
//         Bloom::new(),
//         Tonemapping::new(),
//         Fxaa::new(),
//         EndMainPassPostProcessing::new(),
//         Upscaling::new(),
//     ];

//     PipelineSequence::new(
//         CORE_2D,
//         node_sequence
//             .into_iter()
//             .filter(|x| settings.test_core_sequence_inclusion(x))
//             .collect(),
//     )
// }

/// Creates the default Core 2D rendering pipeline. It consists of the following nodes in sequence:
///
/// [`MsaaWriteback`] `->` [`MainPass`]
pub fn create_msaa_writeback_sequence(settings: Core2dPipelineSettings) -> PipelineSequence {
    let node_sequence: Vec<DynamicPipelineNode> = vec![MsaaWriteback::new()];

    PipelineSequence::new(MSAA_WRITEBACK, node_sequence)
}

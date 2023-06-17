use std::fmt::Display;

use bevy_app::App;
use bevy_render::render_graph::{Node, RenderGraphApp};

/// A helper trait for a [`Node`] implementor that allows it to be used easily for the
/// creation of "pipelines", which are render graphs specialized to a purpose such as
/// 2D rendering, 3D rendering, and so on.  .  
pub trait PipelineNode: Display + Clone + Copy {
    /// The [`Node`] implementor specifying what this pipeline node will do.
    type AssociatedNode: Node;

    /// Label of this node.
    fn label(&self) -> &'static str;

    /// Adds the underlying [`Node`] of interest to the graph of the rendering app.
    fn add_node(&self, sub_graph_name: &str, render_app: &mut App) {
        render_app.add_render_graph_node::<Self::AssociatedNode>(sub_graph_name, self.to_string());
    }
}

/// An sequence of [`PipelineNode`]s that will be connected by edges that mirror the sequence order.
///
/// Usually such a sequence of nodes makes up the primary trunk of a render graph.
pub struct PipelineSequence<N: PipelineNode>(Vec<N>);

impl<N: PipelineNode> PipelineSequence<N> {
    /// Create a new sequence from a slice of [`PipelineNode`] implementors.
    fn new(raw_sequence: &[N]) -> Result<PipelineSequence<N>, String> {
        PipelineSequence(raw_sequence.collect())
    }

    /// Get the labels of each [`PipelineNode`] in this sequence.
    fn labels(&self) -> Vec<&'static str> {
        self.0
            .iter()
            .map(|pipeline_node| pipeline_node.into())
            .collect()
    }

    fn insert_sequence(&self, sub_graph_name: &str, render_app: &mut App) {
        render_app.add_render_sub_graph(sub_graph_name);
        let mut labels = Vec::with_capacity(self.0.len());
        for labelled_node in self.0.iter() {
            labelled_node.add_node(sub_graph_name, render_app);
            labels.push(labelled_node.label());
        }
        render_app.add_render_graph_edges(sub_graph_name, &labels);
    }
}

// render_app
//     .add_render_sub_graph(CORE_2D)
//     .add_render_graph_node::<MainPass2dNode>(CORE_2D, MAIN_PASS)
//     .add_render_graph_node::<ViewNodeRunner<TonemappingNode>>(CORE_2D, TONEMAPPING)
//     .add_render_graph_node::<EmptyNode>(CORE_2D, END_MAIN_PASS_POST_PROCESSING)
//     .add_render_graph_node::<ViewNodeRunner<UpscalingNode>>(CORE_2D, UPSCALING)
//     .add_render_graph_edges(
//         CORE_2D,
//         &[
//             MAIN_PASS,
//             TONEMAPPING,
//             END_MAIN_PASS_POST_PROCESSING,
//             UPSCALING,
//         ],
//     );

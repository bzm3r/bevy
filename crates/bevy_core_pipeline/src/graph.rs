use std::fmt::Display;

use bevy_app::App;
use bevy_render::render_graph::{Node, RenderGraphApp};

/// A helper trait for a [`Node`] implementor that allows it to be used easily for the
/// creation of "pipelines", which are render graphs specialized to a purpose such as
/// 2D rendering, 3D rendering, and so on.
pub trait PipelineNode: Display + Clone + Copy {
    /// The underlying [`Node`] that will be inserted by this [`PipelineNode`] into the render graph.
    type NODE: Default + Node;
    /// The name given to the render graph node that will be inserted by this [`PipelineNode`].
    const NODE_NAME: &'static str;

    /// Adds [`NODE`](Self::NODE) to specified sub graph of the rendering app.
    fn add_node(&self, render_app: &mut App, sub_graph_name: &str) {
        render_app.add_render_graph_node::<Self::NODE>(sub_graph_name, Self::NODE_NAME);
    }
}

/// Facilitates implementation of [`PipelineNode`].
///
/// You can use it in the following ways:
///
/// 1. By providing the identifier of the [`PipelineNode`] implementor, the underlying [`NODE`](PipelineNode::NODE)
/// it represents, and its [`NODE_NAME`](PipelineNode::NODE_NAME):
/// ```
/// use crate::tonemapping::node::TonemappingNode;
///
/// struct CustomPipelineTonemapping;
///
/// pipeline_node!(CustomPipelineTonemapping, TonemappingNode, "tonemapping");
/// ```
///
/// 2. By providing only the identifier of the [`PipelineNode`] implementor and its underlying
/// [`NODE`](PipelineNode::NODE):
/// ```
/// use crate::tonemapping::node::TonemappingNode;
///
/// struct CustomPipelineTonemapping;
///
/// pipeline_node!(CustomPipelineTonemapping, TonemappingNode);
/// ```
/// This will set [`NODE_NAME`](PipelineNode::NODE_NAME) to be the same as the identifier of the [`PipelineNode`]
/// implementor; in this case: "CustomPipelineTonemapping".
#[macro_export]
macro_rules! pipeline_node {
    ( $pipeline_node:ident, $node:ident, $node_name:literal ) => {
        impl PipelineNode for $pipeline_node {
            type NODE = $node;
            const NODE_NAME = $node_name;
        }
    };
    ( $pipeline_node:ident, $node:ident ) => {
        impl PipelineNode for $pipeline_node {
            type NODE = $node;
            const NODE_NAME = stringify!($pipeline_node);
        }
     }
}

/// An sequence of [`PipelineNode`]s that will be connected by edges that mirror the sequence order.
pub struct PipelineSequence(Vec<Box<dyn PipelineNode<NODE = dyn Node>>>);

impl PipelineSequence {
    /// Create a new sequence from a slice of [`PipelineNode`] implementors.
    fn new(raw_sequence: &[Box<Box<dyn PipelineNode<NODE = dyn Node>>>]) -> PipelineSequence {
        PipelineSequence(raw_sequence.iter().copied().collect())
    }

    /// Get the labels of each [`PipelineNode`] in this sequence.
    fn connection_sequence(&self) -> Vec<&'static str> {
        self.0
            .iter()
            .map(|pipeline_node| pipeline_node::NODE_NAME)
            .collect()
    }

    /// Insert this pipeline sequence as a new sub-graph of the [`RenderGraph`](bevy::render::render_graph::RenderGraph)
    /// of the supplied render [`App`].
    fn create_new_sub_graph(&self, render_app: &mut App, sub_graph_name: &str) {
        render_app.add_render_sub_graph(sub_graph_name);
        self.insert_into_sub_graph(render_app, sub_graph_name)
    }

    /// Insert this pipeline sequence into an existing sub-graph of the
    /// [`RenderGraph`](bevy::render::render_graph::RenderGraph) of the supplied render [`App`].
    fn insert_into_sub_graph(&self, render_app: &mut App, sub_graph_name: &str) {
        for pipeline_node in self.0.iter() {
            pipeline_node.add_node(sub_graph_name, render_app);
        }
        render_app.add_render_graph_edges(sub_graph_name, &self.connection_sequence());
    }
}

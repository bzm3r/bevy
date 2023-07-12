use bevy_app::App;
use bevy_render::render_graph::RenderGraphApp;

use super::abstract_node::AbstractNode;

/// Syntactic sugar to facilitate code readability.
pub type DynAbstractNode = Box<dyn AbstractNode>;

/// Trait for an object that generates graphs that are linear sequences.
pub trait LinearGraphGenerator {
    /// Label of the sub-graph that will created by this generator.
    fn graph_label(&self) -> &'static str;

    /// Generate a linear graph object's stored node sequence to create a new sub-graph in the
    /// given render [`App`]'s [`RenderGraph`](bevy::render::render_graph::RenderGraph).
    fn generate_new(&self, render_app: &mut App, sub_graph_name: &str) {
        render_app.add_render_sub_graph(sub_graph_name);
        self.generate_into_existing(
            render_app,
            sub_graph_name,
            Option::<&str>::None,
            Option::<&str>::None,
        );
    }

    /// Generate a linear graph object's stored node sequence to create a new sub-graph in the
    /// given render [`App`]'s [`RenderGraph`](bevy::render::render_graph::RenderGraph).
    ///
    /// An optional `existing_source` (a label for a node in the existing sub graph) can be specified as the
    /// source node for the first node of the generated subgraph. Similarly, an optional `existing_target`
    /// can be specified as the target node of the last node.
    fn generate_into_existing(
        &self,
        render_app: &mut App,
        sub_graph_name: &str,
        existing_source: Option<&'static str>,
        existing_target: Option<&'static str>,
    ) {
        for abstract_node in self.node_sequence.iter() {
            abstract_node.add_node(render_app, sub_graph_name);
        }
        render_app.add_render_graph_edges(
            sub_graph_name,
            existing_source
                .into_iter()
                .chain(self.label_sequence.into_iter())
                .chain(existing_target.into_iter())
                .collect(),
        );
    }
}

// impl LinearGraphGenerator {
//     /// Create a linear sequence from a vector of [`AbstractNode`] implementors.
//     // pub fn new(
//     //     graph_label: &'static str,
//     //     node_sequence: Vec<DynAbstractNode>,
//     // ) -> LinearGraphGenerator {
//     //     let label_sequence = node_sequence.iter().map(|n| n.label()).collect();
//     //     LinearGraphGenerator {
//     //         graph_label,
//     //         node_sequence,
//     //         label_sequence,
//     //     }
//     // }
// }

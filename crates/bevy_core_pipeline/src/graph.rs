use std::{fmt::Display, iter::once, marker::PhantomData};

use bevy_app::App;
use bevy_ecs::world::{FromWorld, World};
use bevy_render::render_graph::{Node, NodeLabel, RenderGraphApp};

pub trait NodeCreator {
    fn create_node(&self, world: &mut World) -> Box<dyn Node>;
}

#[derive(Default, Clone, Copy)]
pub struct NodeFactory<N: Node + FromWorld> {
    node_type: PhantomData<N>,
}

impl<N: Node + FromWorld> NodeCreator for NodeFactory<N> {
    fn create_node(&self, world: &mut World) -> Box<dyn Node> {
        Box::new(N::from_world(world))
    }
}

/// A helper trait for a [`Node`] implementor that allows it to be used easily for the
/// creation of "pipelines", which are render graphs specialized to a purpose such as
/// 2D rendering, 3D rendering, and so on.
///
/// The [`pipeline_node!`](pipeline_node) macro allows for quick creation of a [`PipelineNode`]
/// implementor with all the required trait implementations.
///
/// [`Display`] is a [supertrait](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-supertraits-to-require-one-traits-functionality-within-another-trait)
/// because the [`Display`] implementation species the name of the pipeline node within the render graph.  
pub trait PipelineNode: Display {
    type Factory: NodeCreator;

    /// An object safe [`Node`] creator.
    fn node_factory(&self) -> Self::Factory;

    /// Adds [`NODE`](Self::NODE) to specified sub graph of the rendering app.
    fn add_node(&self, render_app: &mut App, sub_graph_name: &str) {
        let node = self.node_factory().create_node(&mut render_app.world);
        render_app.add_node_to_render_graph(sub_graph_name, self.to_string(), node);
    }
}

/// Implements [`Display`] for an object depending on the parameters supplied.
/// It is primarily a helper for the [`pipeline_node!`](crate::pipeline_node)
/// macro, but could also be used to quickly implement [`Display`] for
/// objects with user-defined implementations of [`PipelineNode`].
///
/// There are two ways to use this macro.
/// 1. Supply the identifier of the object of interest and a string literal label
/// that should be emitted when this object is [`Display]`ed. For example:
/// ```
/// impl_display!(CustomPipelineTonemapping, "tonemapping");
/// ```
/// Then this will expand to:
/// ```
/// impl Display for CustomPipelineTonemapping {
///     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
///         write!(f, "{}", "tonemapping")
///     }
/// }
/// ```
///
/// 2. Supply only the identifier of the object of interest. In this case, the macro
/// assumes that the object implements [`Debug`], and will use the [`Debug`] implementation
/// as the
#[macro_export]
macro_rules! impl_display {
    ( $pipeline_node:ident, $node_node: literal ) => {
        impl std::fmt::Display for $pipeline_node {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(f, "{}", $node_name:literal)
            }
        }
    };
    ( $pipeline_node:ident ) => {
        impl Display for $pipeline_node {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(f, "{:?}", self)
            }
        }
    };
}

/// Allows for quick creation of a `struct` which is a [`PipelineNode`].
///
/// You can use it in the following ways:
///
/// 1. By providing the identifier of the [`PipelineNode`] implementor, the underlying [`NODE`](PipelineNode::NODE)
/// it represents, and a string literal that will be used to [`Display`] the `struct`:
///
/// ```
/// use crate::tonemapping::node::TonemappingNode;
///
/// pipeline_node!(CustomPipelineTonemapping, TonemappingNode, "tonemapping");
/// ```
///
/// This will expand to the following:
///
/// ```
/// #[derive(Clone, Copy, Debug)]
/// pub struct CustomPipelineTonemapping;
///
/// impl Display for CustomPipelineTonemapping {
///     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
///         write!(f, "{}", "tonemapping")
///     }
/// }
///
/// impl PipelineNode for CustomPipelineTonemapping {
///     type NODE = TonemappingNode;
/// }
/// ```
///
/// 2. By providing only the identifier of the [`PipelineNode`] implementor and its underlying
/// [`NODE`](PipelineNode::NODE):
///
/// ```
/// use crate::tonemapping::node::TonemappingNode;
///
/// pipeline_node!(CustomPipelineTonemapping, TonemappingNode);
/// ```
///
/// This will implement [`Display`] using the implementor's [`Debug`] trait. So, the macro will expand the
///  above example to:
///
/// ```
/// #[derive(Clone, Copy, Debug)]
/// pub struct CustomPipelineTonemapping;
///
/// impl Display for CustomPipelineTonemapping {
///     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
///         write!(f, "{:?}", self)
///     }
/// }
///
/// impl PipelineNode for CustomPipelineTonemapping {
///     type NODE = TonemappingNode;
/// }
/// ```
#[macro_export]
macro_rules! pipeline_node {
    ( $pipeline_node:ident, $node:ident $(, $node_name:literal)? ) => {
        #[derive(Default, Clone, Copy, Debug)]
        pub struct $pipeline_node {
            node_factory: NodeFactory<$node>,
        };

        impl_display!($pipeline_node $(, $node_name)?)

        impl PipelineNode for $pipeline_node {
            type Factory = NodeFactory<$node>;

            fn node_factory(&self) -> &Self::Factory {
                &self.node_factory
            }
        }
    };
}

/// Helpful shorthand for making code more readable.
pub type DynamicPipelineNode = Box<dyn PipelineNode<Factory = dyn NodeCreator>>;

/// An sequence of [`PipelineNode`]s that will be connected by edges that mirror the sequence order.
pub struct PipelineSequence {
    node_sequence: Vec<DynamicPipelineNode>,
    label_sequence: Vec<NodeLabel>,
}

impl PipelineSequence {
    /// Create a new sequence from a vector of [`PipelineNode`] implementors.
    fn new(node_sequence: Vec<DynamicPipelineNode>) -> PipelineSequence {
        let label_sequence = node_sequence.iter().map(|n| n.to_string().into()).collect();
        PipelineSequence {
            node_sequence,
            label_sequence,
        }
    }

    /// Use this pipeline sequence to create a new sub-graph of the
    /// [`RenderGraph`](bevy::render::render_graph::RenderGraph) of the supplied render [`App`].
    fn create_new_sub_graph(&self, render_app: &mut App, sub_graph_name: &str) {
        render_app.add_render_sub_graph(sub_graph_name);
        self.insert_into_sub_graph(render_app, sub_graph_name, Option::<&str>::None);
    }

    /// Insert this pipeline sequence into an existing sub-graph of the
    /// [`RenderGraph`](bevy::render::render_graph::RenderGraph) of the supplied render [`App`].
    ///
    /// An optional existing node, `existing_root`, can be connected to the first node
    /// in this pipeline sequence.
    fn insert_into_sub_graph(
        &self,
        render_app: &mut App,
        sub_graph_name: &str,
        existing_root: Option<impl Into<NodeLabel>>,
    ) {
        for pipeline_node in self.node_sequence.iter() {
            pipeline_node.add_node(render_app, sub_graph_name);
        }
        render_app.add_render_graph_edges(
            sub_graph_name,
            match existing_root {
                Some(existing_root) => once(existing_root.into())
                    .chain(self.label_sequence.clone().into_iter())
                    .collect(),
                None => self.label_sequence.clone(),
            },
        );
    }
}

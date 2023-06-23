use bevy_app::App;
use bevy_ecs::world::{FromWorld, World};
use bevy_render::render_graph::{Node, RenderGraphApp};
use std::fmt::{Debug, Error, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;

pub trait NodeCreator {
    fn create_node(&self, world: &mut World) -> Box<dyn Node>;
}

impl NodeCreator for Box<dyn NodeCreator> {
    fn create_node(&self, world: &mut World) -> Box<dyn Node> {
        self.deref().create_node(world)
    }
}

pub struct NodeFactory<N: Node + FromWorld> {
    pub node_type: PhantomData<N>,
}

impl<N: Node + FromWorld> Default for NodeFactory<N> {
    fn default() -> NodeFactory<N> {
        NodeFactory {
            node_type: PhantomData,
        }
    }
}

impl<N: Node + FromWorld> Clone for NodeFactory<N> {
    fn clone(&self) -> Self {
        NodeFactory {
            node_type: self.node_type.clone(),
        }
    }
}

impl<N: Node + FromWorld> Copy for NodeFactory<N> {}

impl<N: Node + FromWorld> Debug for NodeFactory<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "NodeFactory {{ node_type: {:?} }}", self.node_type)
    }
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
/// The [`pipeline_node!`](pipeline_node) macro allows for quick creation of a convenient
/// [`PipelineNode`] implementor.
pub trait PipelineNode {
    type Factory: NodeCreator;

    /// The label of this pipeline node in the render graph.
    fn label(&self) -> &'static str;

    /// An object safe [`Node`] creator.
    fn node_factory(&self) -> Self::Factory;

    /// Adds [`NODE`](Self::NODE) to specified sub graph of the rendering app.
    fn add_node(&self, render_app: &mut App, sub_graph_name: &str) {
        let node = self.node_factory().create_node(&mut render_app.world);
        render_app.add_node_to_render_graph(sub_graph_name, self.label(), node);
    }
}

/// Helper macro for initializing the default value of the `label: String` field on a struct generated
/// using the [`pipeline_node!`](pipeline_node).declarative macro.
#[macro_export]
macro_rules! generate_default_pipeline_node_label {
    ( $pipeline_node:ident, $label: literal ) => {
        $label
    };
    ( $pipeline_node:ident ) => {
        stringify!(paste! {
            [<$pipeline_node:snake:lower>]
        })
    };
}

/// Allows for quick creation of a `struct` which is a [`PipelineNode`].
///
/// You can use it in the following ways:
///
/// 1. By providing the identifier of the [`PipelineNode`] implementor, the underlying [`NODE`](PipelineNode::NODE)
/// it represents, and a string literal (`&'static str`) that will be used as the pipeline node's label
/// and a string identifier. For example with an identifier `CustomPipelineTonemapping`, a
/// node type `TonemappingNode`, and a string literal `"tonemapping"`:
/// ```
/// use crate::tonemapping::node::TonemappingNode;
///
/// pipeline_node!(CustomPipelineTonemapping, TonemappingNode, "tonemapping");
/// ```
/// 2. By providing only the identifier of the [`PipelineNode`] implementor and its underlying
/// [`NODE`](PipelineNode::NODE). For example with an identifier `CustomPipelineTonemapping` and a
/// node type `TonemappingNode`:
/// ```
/// use crate::tonemapping::node::TonemappingNode;
///
/// pipeline_node!(CustomPipelineTonemapping, TonemappingNode);
/// ```
///
/// In either case, the following is generated:
/// ```
/// /// Auto-generated struct, using [`pipeline_node!](bevy_core_pipeline::pipelining::pipeline_node).
/// #[derive(Clone, Copy, Debug)]
/// pub struct CustomPipelineTonemapping {
///     label: Cow<'static, str>,
///     node_factory: NodeFactory<TonemappingNode>
/// }
///
/// impl CustomPipelineTonemapping {
///     fn new(label: Into<Cow<'static, str>>) -> Self {
///         struct CustomPipelineTonemapping {
///             label,
///             node_factory: NodeFactory::<TonemappingNode>::default(),
///         }
///     }
/// }
///
/// impl PipelineNode for CustomPipelineTonemapping {
///     type Factory = NodeFactory<Tonemapping>;
///
///     fn label(&self) -> &str {
///         self.label.deref()
///     }
///
///     fn clone_label(&self) -> Cow<'static, str> {
///         self.label.clone()
///     }
///
///     fn node_factory(&self) -> &Self::Factory {
///         &self.node_factory
///     }
/// }
/// ```
///
/// Depending on how the macro was called, one of the following is generated:
/// ```
/// // string label was provided by user
/// impl Default for CustomPipelineTonemapping {
///     fn default() -> Self {
///         struct CustomPipelineTonemapping {
///             label: "tonemapping".into(),
///             node_factory: NodeFactory::<$node>::default(),
///         }
///     }
/// }
/// ```
///
/// ```
/// // string label was NOT provided by user, the struct's name is literalized as the label
/// impl Default for CustomPipelineTonemapping {
///     fn default() -> Self {
///         struct CustomPipelineTonemapping {
///             label: "CustomPipelineTonemapping".into(),
///             node_factory: NodeFactory::<$node>::default(),
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! pipeline_node {
    ( $pipeline_node:ident, $node:ty $(, $label:literal)? ) => {
        paste! {
            pub const [<$pipeline_node:snake:upper>]: &str = $crate::pipelining::generate_default_pipeline_node_label!($pipeline_node $(, $label:literal)?);
        }
        /// Auto-generated struct, using [`pipeline_node!](bevy_core_pipeline::pipelining::pipeline_node).
        ///
        /// This is almost always use boxed, so the `new` method returns a `Box<Self>` instead of `Self`.
        #[derive(Clone, Debug)]
        pub struct $pipeline_node {
            node_factory: std::boxed::Box<$crate::pipelining::NodeFactory<$node>>,
        }

        impl $pipeline_node {
            fn new() -> std::boxed::Box<Self> {
                std::boxed::Box::new($pipeline_node {
                    node_factory: std::boxed::Box::new($crate::pipelining::NodeFactory::<$node>::default()),
                })
            }
        }

        impl $crate::pipelining::PipelineNode for $pipeline_node {
            type Factory = Box<dyn $crate::pipelining::NodeCreator>;

            /// Get the label of this pipeline node.
            fn label(&self) -> &'static str {
                $crate::pipelining::generate_default_pipeline_node_label!($pipeline_node $(, $label:literal)?)
            }

            /// Get the node factory for this pipeline node.
            fn node_factory(&self) -> Self::Factory {
                self.node_factory
            }
        }
    };
}

#[macro_export]
macro_rules! pipeline_nodes {
    ( $(($tokens:tt),)* ) => {
        $( pipeline_node!($tokens) )*
    }
}

/// Helpful shorthand for making code more readable.
pub type DynamicPipelineNode = Box<dyn PipelineNode<Factory = Box<dyn NodeCreator>>>;

/// An sequence of [`PipelineNode`]s that will be connected by edges that mirror the sequence order.
pub struct PipelineSequence {
    pipeline_label: &'static str,
    node_sequence: Vec<DynamicPipelineNode>,
    label_sequence: Vec<&'static str>,
}

impl PipelineSequence {
    /// Create a new sequence from a vector of [`PipelineNode`] implementors.
    pub fn new(
        pipeline_label: &'static str,
        node_sequence: Vec<DynamicPipelineNode>,
    ) -> PipelineSequence {
        let label_sequence = node_sequence.iter().map(|n| n.label()).collect();
        PipelineSequence {
            pipeline_label,
            node_sequence,
            label_sequence,
        }
    }

    /// Use this pipeline sequence to create a new sub-graph of the
    /// [`RenderGraph`](bevy::render::render_graph::RenderGraph) of the supplied render [`App`].
    pub fn create_new_sub_graph(&self, render_app: &mut App, sub_graph_name: &str) {
        render_app.add_render_sub_graph(sub_graph_name);
        self.insert_into_sub_graph(
            render_app,
            sub_graph_name,
            Option::<&str>::None,
            Option::<&str>::None,
        );
    }

    /// Insert this pipeline sequence into an existing sub-graph of the
    /// [`RenderGraph`](bevy::render::render_graph::RenderGraph) of the supplied render [`App`].
    ///
    /// An optional existing node, `existing_root`, can be connected to the first node
    /// in this pipeline sequence. `existing_target` is similar, but is connected to the last
    /// node of this pipeline sequence.  
    pub fn insert_into_sub_graph(
        &self,
        render_app: &mut App,
        sub_graph_name: &str,
        existing_root: Option<&'static str>,
        existing_target: Option<&'static str>,
    ) {
        for pipeline_node in self.node_sequence.iter() {
            pipeline_node.add_node(render_app, sub_graph_name);
        }
        render_app.add_render_graph_edges(
            sub_graph_name,
            existing_root
                .into_iter()
                .chain(self.label_sequence.clone().into_iter())
                .chain(existing_target.into_iter())
                .collect(),
        );
    }
}

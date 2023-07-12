use bevy_app::App;
use bevy_ecs::world::{FromWorld, World};
use bevy_render::render_graph::Node;
use std::fmt::Debug;
use std::marker::PhantomData;

use paste::paste;

/// Helper for creating the label applied to concrete nodes created by an [`AbstractNode`] implementor.
#[macro_export]
macro_rules! generate_default_abstract_node_label {
    ( 
        $abstract_node:ident, 
        $label: literal 
    ) => {
        $label
    };
    ( $abstract_node:ident ) => {
        stringify!(
            paste! {
                [< $abstract_node:snake:lower >]
            }
        )
    };
}

/// A structure aware at compile-time of the type of [`Node`] implementor it should create.
/// 
/// Note that the [`Node`] implementor must also implement [`FromWorld`].
#[derive(Clone, Copy, Default, Debug)]
pub struct NodeFactory<N: Node + FromWorld> {
    pub node_type: PhantomData<N>,
}

impl<N: Node + FromWorld> NodeFactory<N> {
    fn create(&self, world: &mut World) -> Box<dyn Node> {
        Box::new(N::from_world(world))
    }
}

// pub struct AbstractNode<N: Node + FromWorld> {
//     pub node_type: PhantomData<N>,
// }

// Box::new(N::from_world(world))

/// Marks objects which facilitate run-time creation of [`Node`]s for render graphs.
/// 
/// This trait is object safe, but making it so requires that it .
///
/// The [`abstract_node!`](abstract_node) macro allows for quick creation of a convenient
/// [`AbstractNode`] implementor.
pub trait AbstractNode {
    /// The in-graph label of the concrete node crated.
    fn label(&self) -> &'static str;


    /// Creates a boxed, concrete object of the target [`Node`]'s type. 
    fn create(&self, world: &mut World) -> Box<dyn Node>;

    /// Adds [`NODE`](Self::NODE) to specified sub graph of the rendering app.
    fn insert_concrete(&self, render_app: &mut App, sub_graph_name: &str) {
        let node = self.create_concrete(&mut render_app.world);
        render_app.add_node_to_render_graph(sub_graph_name, self.label(), node);
    }
}

/// Helper for defining a [`AbstractNode`] implementor.
/// 
/// It takes two required and one optional comma-separated arguments, in the following order:
///     1. a `CamelCase` [`ident`](https://doc.rust-lang.org/reference/macros-by-example.html#metavariables) (type) 
/// of the structure generated by this macro (`abstract_id`);
///     2. a `CamelCase` concrete type of the [`Node`] implementor that can be created by this structure (`concrete_ty`);
///     3. (optional) a string literal used as a label for the concrete node once inserted into a render graph (`label`). 
/// If a label is not given, one will be generated by converting the SnakeCase `abstract_id` into a `snake_case` string
/// literal. 
/// 
/// For example:
/// 
/// ```rust
/// // creates an abstract structure of type `MainPass`, which will create
/// // a concrete node of type core2d::MainPass2d, with the label "main_pass"
/// abstract_node!(MainPass, core2d::MainPass2d);
/// ```
/// 
/// ```rust
/// // creates an abstract structure of type `MainPass`, which will create
/// // a concrete node of type core2d::MainPass2d, with the label "hello_world"
/// abstract_node!(MainPass, core2d::MainPass2d, "hello_world");
/// ```
#[macro_export]
macro_rules! abstract_node {
    ( 
        $abstract_id:ident, 
        $concrete_ty:ty 
        $(, $label:literal)? 
    ) => {
        #[derive(Default, Clone, Copy)]
        pub struct $abstract_id {
            pub factory: NodeFactory<$concrete_ty>,
        }

        paste! {
            pub const [< $abstract_id:snake:upper >]: &str = 
                $crate::graph_making::abstract_node::
                    default_label!($abstract_id $(, $label:literal)?);
        }

        impl Debug for $abstract_id {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                //TODO: this might not work
                write!(f, "{}<{}>", stringify!($abstract_id), stringify!($concrete_ty))
            }
        }

        impl AbstractNode for $abstract_id {
            fn label(&self) -> &'static str {
                $crate::graph_making::abstract_node::
                    default_label!($abstract_id $(, $label:literal)?)
            }

            fn create(&self, world: &mut World) -> Box<dyn Node> {
                self.factory.create(world)
            }
        }
    }
}

/// Facilitates creation of multiple [`AbstractNode`]s. 
/// 
/// It takes a comma-separated sequence of tuples that are valid arguments for [`abstract_node`].
/// 
/// Under the hood, this expands into multiple [`abstract_node`] calls. For example, these two code snippets are equivalent: 
/// ```rust
/// abstract_nodes!(
///     (Bloom, BloomNode, "bloom_2d"), 
///     (Tonemapping, TonemappingNode)
/// );
/// ```
/// 
/// ```rust
/// abstract_node!(Bloom, BloomNode, "bloom_2d");
/// abstract_node!(Tonemapping, TonemappingNode);
/// ```
#[macro_export]
macro_rules! abstract_nodes {
    ( $(
        (
            $abstract_node:ident, 
            $node:ty 
            $(, $label:literal)?
        )),* 
    ) => {
        $( $crate::graph_making::abstract_node::abstract_node!($abstract_node, $node $(, $label)?); )*
    }
}
use crate::core_2d::{Core2dGenerator, Core2dPlugin};
use paste::paste;

use super::topology::Topology;

/// Trait for an object that can generate a render graph.
pub trait GraphGenerator<G> where G:Topology {}

/// Marker trait for an object containing references to the plugins required to initialize
/// a [`GraphGenerator`] that is [`Dependent`] on information from the generators of other
/// plugins.
pub trait ParentPlugins {}

/// Marker trait for an object containing data that is necessary to initialize a [`GraphGenerator`].
pub trait RequiredFields: Default + Clone {}

/// Trait for a render graph generator which depend on generators from "parent" plugins.
///
/// See [`Independent`] for those that don't.
pub trait Dependent<P, F>
where
    P: ParentPlugins,
    F: RequiredFields,
{
    
    /// Create a new generator.
    fn new(graph_label: &'static str, node_sequence: Vec<DynAbstractNode>) -> Self;
    fn new(parent_plugins: P, required_fields: F) -> Self;
}

/// Trait for graph generator which do not depend on generators from "parent" plugins.
///
/// See [`Dependent`] for those that do.
pub trait Independent<F>
where
    F: RequiredFields,
{
    fn new(graph_label: &'static str, required_fields: F) -> Self;
}

#[macro_export]
macro_rules! create_parent_plugins {
    (
        $name:ident;
        $(
            $([$($doc_string:literal),*])?
            $required:ident
        ),+
    ) => {
        paste! {
            #[doc = "Specifies the plugins required to create [`" [< $name Generator >] "`]: "]
            $(#[doc = "   * [`" [< $required Plugin >] "`]"])+
            pub struct [< $name ParentPlugins >]<'a> {
                $(
                    $(#[doc = $doc_string])*
                    pub [< $required:lower:snake  >]: &'a [< $required Plugin >]
                )?
            }

            impl<'a> ParentPlugins for [< $name ParentPlugins >]<'a> {}
        }
    };
}

#[macro_export]
macro_rules! default_field_initializer {
    ( $concrete_default:expr ) => {
        $concrete_default
    };
    () => {
        Default::default()
    };
}

#[macro_export]
macro_rules! generate_required_fields {
    (
        $name:ident;
        $(
            $([$($doc_string:literal),*])?
            $field_name:ident : $field_ty:ty
            $(= $optional_default:expr)?
        ),+
    ) => {
        paste! {
            #[doc = "Data required to initialize [`" [< $name Generator >] "`]."]
            #[derive(Clone)]
            pub struct [< $name RequiredFields >] {
                $(
                    $([$(#[doc = $doc_string]),*])?
                    pub $field_name: $field_ty
                ),+
            }

            impl Default for [< $name RequiredFields >] {
                fn default() -> Self {
                    Self {
                        $(
                            $field_name: default_field_initializer! {
                                $($optional_default)?
                            }
                        ),+
                    }
                }
            }

            impl RequiredFields for [< $name RequiredFields >] {}
        }
    }
}

#[macro_export]
macro_rules! impl_initialization_trait {
    (
        $name:ident ;
        $(
            $field_name:ident : $field_ty:ty
            $(= $optional_default:expr)?
        ),+ ;
        $(
            $parent_plugin:ident
        ),+
    ) => {
        paste! {
            impl<'a> Dependent<[< $name ParentPlugins >]<'a> , [< $name RequiredFields >]> for [< $name Generator >] {
                fn new(parent_plugins: [< $name ParentPlugins >]<'a>, required_fields: [< $name RequiredFields >]) -> Self {
                    [< $name Generator >] {
                        $([< $parent_plugin:lower:snake > ]: parent_plugins.[< $parent_plugin:lower:snake > ].graph_gen().clone()),+
                        $($field_name: required_fields.$field_name),+
                    }
                }
            }
        }
    };
    (
        $name:ident ;
        $(
            $field_name:ident : $field_ty:ty
            $(= $optional_default:expr)?
        ),+
    ) => {
        paste! {
            impl<'a> Independent<[< $name RequiredFields >]> for [< $name Generator >] {
                fn new(required_fields: [< $name RequiredFields >]) -> Self {
                    [< $name Generator >] {
                        $($field_name: required_fields.$field_name),+
                        $([< $parent_plugin:lower:snake  >]: parent_plugins.[< $parent_plugin:lower:snake >].graph_gen().clone()),+
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! create_graph_generator {
    ($name:ident;  $($([$($field_doc_string:literal),*])? $field_name:ident : $field_ty:ty $(= $optional_default:expr)?),+ $(; $($([$($plugin_doc_string:literal),*])? $parent_plugin:ident),+)? ) => {
        paste! {
            #[doc = "Generator to configure the render graph generated for [`" [< $name Plugin >] "`]."]
            #[doc = ""]
            #[doc = "Initialization requires data for fields specified in [`" [< $name RequiredFields >] "`]."]
            $(
                // We do not want to use $parent_plugin metavariables here, but we do want to only generate the
                // following if they were provided to this macro call. So we "use them", with no effect, in a cfg_attr
                // that with a predicate that evaluates to false (existential quantification over an empty set).
                #[cfg_attr(any(), $($parent_plugin),+)]
                #[doc = ""]
                #[doc = "Initialization also requires settings from plugins specified in [`" [< $name ParentPlugins >] "`]."]
            )?
            #[derive(Clone, Debug, Default)]
            pub struct [< $name Generator >] {
                // $($($parent_plugin:ident),+)?
                $($(pub [< $parent_plugin:lower:snake >]: [< $parent_plugin Generator >],)+)?
                // $($(pub [< $parent_plugin:lower:snake >]: [< $parent_plugin Generator >]),+)?
                $(pub $field_name: $field_ty),+
            }

            generate_required_fields!(
                $name ;
                $(
                    $(
                        [$($field_doc_string),*]
                    )?
                    $field_name : $field_ty $(= $optional_default)?
                ),+
            );

            $(
                create_parent_plugins!(
                    $name ;
                    $(
                        $(
                            [$($plugin_doc_string),*]
                        )?
                        $parent_plugin
                    ),+
                );
            )?

            impl_initialization_trait!(
                $name ;
                $(
                    $field_name : $field_ty
                    $(= $optional_default)?
                ),+
                $(; $($parent_plugin),+ )?
            );
        }
    }
}
create_graph_generator!(MsaaWriteback; my_test: bool = false ; Core2d );

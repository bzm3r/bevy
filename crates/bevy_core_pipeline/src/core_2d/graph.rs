pub const NAME: &str = "core_2d";

pub mod input {
    pub const VIEW_ENTITY: &str = "view_entity";
}

pub mod old_node {
    pub const MSAA_WRITEBACK: &str = "msaa_writeback";
    pub const MAIN_PASS: &str = "main_pass";
    pub const BLOOM: &str = "bloom";
    pub const TONEMAPPING: &str = "tonemapping";
    pub const FXAA: &str = "fxaa";
    pub const UPSCALING: &str = "upscaling";
    pub const CONTRAST_ADAPTIVE_SHARPENING: &str = "contrast_adaptive_sharpening";
    pub const END_MAIN_PASS_POST_PROCESSING: &str = "end_main_pass_post_processing";
}

#[derive(Debug, Clone, Copy)]
pub enum Core2dNode {
    MsaaWriteback,
    MainPass,
    Bloom,
    Tonemapping,
    Fxaa,
    Upscaling,
    ContrastAdaptiveSharpening,
    EndMainPassPostProcessing,
}

impl Display for Core2dNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl PipelineNode for Core2dNode {
    type AssociatedNode = ;
    fn add_node(&self, sub_graph_name: &str, render_app: &mut bevy_app::App) {
        match self {
            MsaaWriteback => {
                render_app
                    .add_render_graph_node::<MsaaWritebackNode>(
                        sub_graph_name,
                        Core2dNode::MsaaWriteback.label(),
                    )
                    .add_render_graph_edge(
                        sub_graph_name,
                        self.label(),
                        Core2dNode::MainPass.label(),
                    );
            }
            MainPass => {
                render_app.add_render_graph_node::<MainPass2dNode>(sub_graph_name, self.label())
            }
            Bloom => {
                render_app.add_render_graph_node::<MainPass2dNode>(sub_graph_name, self.label())
            }
            Tonemapping => {
                render_app.add_render_graph_node::<MainPass2dNode>(sub_graph_name, self.label())
            }
            Fxaa => {
                render_app.add_render_graph_node::<MainPass2dNode>(sub_graph_name, self.label())
            }
            Upscaling => {
                render_app.add_render_graph_node::<MainPass2dNode>(sub_graph_name, self.label())
            }
            ContrastAdaptiveSharpening => {
                render_app.add_render_graph_node::<MainPass2dNode>(sub_graph_name, self.label())
            }
            EndMainPassPostProcessing => {
                render_app.add_render_graph_node::<MainPass2dNode>(sub_graph_name, self.label())
            }
        }
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

use std::fmt::Display;

use Core2dNode::*;

use crate::{graph::PipelineNode, msaa_writeback::MsaaWritebackNode};

use super::MainPass2dNode;
pub const DEFAULT: [Core2dNode; 4] = [MainPass, Tonemapping, EndMainPassPostProcessing, Upscaling];

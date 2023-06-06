#![allow(clippy::type_complexity)]

pub mod blit;
pub mod clear_color;
pub mod fullscreen_vertex_shader;
pub mod prepass;
mod skybox;
pub use skybox::Skybox;
pub mod camera2d;
pub mod camera3d;

#[cfg(feature = "bloom")]
pub mod bloom;

#[cfg(feature = "casp")]
pub mod contrast_adaptive_sharpening;
#[cfg(feature = "core2d")]
pub mod core_2d;

#[cfg(feature = "core3d")]
pub mod core_3d;
#[cfg(feature = "core3d")]
mod taa;

#[cfg(feature = "fxaa")]
pub mod fxaa;
#[cfg(feature = "msaa_writeback")]
pub mod msaa_writeback;
#[cfg(feature = "tonemapping")]
pub mod tonemapping;
#[cfg(feature = "tonemapping")]
pub mod upscaling;

/// Experimental features that are not yet finished. Please report any issues you encounter!
pub mod experimental {
    #[cfg(feature = "core3d")]
    pub mod taa {
        pub use crate::taa::*;
    }
}

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        camera2d::{Camera2d, Camera2dBundle},
        camera3d::{Camera3d, Camera3dBundle},
        clear_color::ClearColor,
    };
}

use crate::{
    blit::BlitPlugin,
    bloom::BloomPlugin,
    clear_color::{ClearColor, ClearColorConfig},
    contrast_adaptive_sharpening::CASPlugin,
    core_2d::Core2dPlugin,
    core_3d::Core3dPlugin,
    fullscreen_vertex_shader::FULLSCREEN_SHADER_HANDLE,
    fxaa::FxaaPlugin,
    msaa_writeback::MsaaWritebackPlugin,
    prepass::{DepthPrepass, NormalPrepass},
    tonemapping::TonemappingPlugin,
    upscaling::UpscalingPlugin,
};
use bevy_app::{App, Plugin};
use bevy_asset::load_internal_asset;
use bevy_render::{extract_resource::ExtractResourcePlugin, prelude::Shader};

#[derive(Default)]
pub struct CorePipelinePlugin;

impl Plugin for CorePipelinePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            FULLSCREEN_SHADER_HANDLE,
            "fullscreen_vertex_shader/fullscreen.wgsl",
            Shader::from_wgsl
        );

        app.register_type::<ClearColor>()
            .register_type::<ClearColorConfig>()
            .register_type::<DepthPrepass>()
            .register_type::<NormalPrepass>()
            .init_resource::<ClearColor>()
            .add_plugin(ExtractResourcePlugin::<ClearColor>::default())
            .add_plugin(BlitPlugin);
        #[cfg(feature = "core2d")]
        app.add_plugin(Core2dPlugin);
        #[cfg(feature = "core3d")]
        app.add_plugin(Core3dPlugin);
        #[cfg(feature = "msaa_writeback")]
        app.add_plugin(MsaaWritebackPlugin);
        #[cfg(feature = "tonemapping")]
        app.add_plugin(TonemappingPlugin);
        #[cfg(feature = "upscaling")]
        app.add_plugin(UpscalingPlugin);
        #[cfg(feature = "bloom")]
        app.add_plugin(BloomPlugin);
        #[cfg(feature = "fxaa")]
        app.add_plugin(FxaaPlugin);
        #[cfg(feature = "casp")]
        app.add_plugin(CASPlugin);
    }
}

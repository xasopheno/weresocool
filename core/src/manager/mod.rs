mod audio_engine;
mod buffer_manager;
mod midi_controller;
mod render_manager;
mod resizeable_2d_vec;
mod visualization_adapter;
mod volume_controller;

#[allow(deprecated)]
pub use self::{
    midi_controller::{MidiClient, MidiController},
    render_manager::{
        prepare_render_outside, KillChannel, RenderManager,
        RenderManagerSettings, VisEvent,
    },
    resizeable_2d_vec::Resizeable2DVec,
    visualization_adapter::VisualizationAdapter,
    volume_controller::VolumeController,
};

// Re-export visualization functions for backward compatibility
pub fn render_op_to_normalized_op4d(
    render_op: &weresocool_instrument::renderable::RenderOp,
    normalizer: &crate::generation::Normalizer,
) -> Option<crate::generation::Op4D> {
    VisualizationAdapter::render_op_to_normalized_op4d(render_op, normalizer)
}

pub fn render_op_to_normalized_op4d_list(
    render_op: &weresocool_instrument::renderable::RenderOp,
    normalizer: &crate::generation::Normalizer,
    frame_length: f64,
) -> Vec<crate::generation::Op4D> {
    VisualizationAdapter::render_op_to_normalized_op4d_list(render_op, normalizer, frame_length)
}

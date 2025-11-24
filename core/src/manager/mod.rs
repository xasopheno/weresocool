mod buffer_manager;
mod midi_controller;
mod render_manager;
mod resizeable_2d_vec;
mod volume_controller;

#[allow(deprecated)]
pub use self::{
    midi_controller::{MidiClient, MidiController},
    render_manager::{
        prepare_render_outside, render_op_to_normalized_op4d, KillChannel, RenderManager,
        RenderManagerSettings, VisEvent, render_op_to_normalized_op4d_list
    },
    resizeable_2d_vec::Resizeable2DVec,
    volume_controller::VolumeController,
};

/// Volume Controller for WereSoCool
///
/// Manages volume ramping and crossfading for smooth audio transitions.

use crate::events::{Events, StateEvent};

#[derive(Debug, Clone)]
pub struct VolumeController {
    current_volume: f32,
    past_volume: f32,
}

impl VolumeController {
    pub fn new() -> Self {
        Self {
            current_volume: 0.8,
            past_volume: 0.8,
        }
    }

    /// Update the target volume and emit state event
    pub fn update_volume(&mut self, volume: f32, events: &mut Events) {
        self.current_volume = f32::powf(volume, 2.0);
        if events.state.has_subscribers() {
            events.state.emit(StateEvent::Volume(volume));
        }
    }

    /// Generate volume ramp from past to current volume
    ///
    /// Creates a smooth crossfade from past_volume to current_volume over the buffer.
    /// Uses longer crossfade for large volume changes to avoid clicks.
    pub fn ramp_to_current_volume(&mut self, buffer_size: usize) -> Vec<f32> {
        let mut offset: Vec<f32> = Vec::with_capacity(buffer_size * 2);
        let distance = self.current_volume - self.past_volume;

        // Use crossfade_period for smoother volume transitions
        let crossfade_samples = weresocool_shared::Settings::global().crossfade_period;

        // If we're far from target volume, use longer crossfade
        let ramp_length = if distance.abs() > 0.3 {
            crossfade_samples.max(buffer_size * 2)
        } else {
            buffer_size * 2
        };

        let denom = ramp_length as f32;
        for i in 0..(buffer_size * 2) {
            if i < ramp_length {
                offset.push(self.past_volume + (distance * i as f32 / denom));
            } else {
                offset.push(self.current_volume);
            }
        }

        // Only update past_volume if we've reached the target
        if buffer_size * 2 >= ramp_length {
            self.past_volume = self.current_volume;
        } else {
            self.past_volume += distance * (buffer_size * 2) as f32 / denom;
        }

        offset
    }

    pub fn current_volume(&self) -> f32 {
        self.current_volume
    }

    pub fn past_volume(&self) -> f32 {
        self.past_volume
    }
}

impl Default for VolumeController {
    fn default() -> Self {
        Self::new()
    }
}

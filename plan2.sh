let frequency = if self.sound_to_silence() {
  self.past.frequency
} else if self.portamento_index < info.portamento_length
    && !self.silence_to_sound()
    && !self.sound_to_silence()
{
    self.past.frequency + (info.index as f64 * info.p_delta)
} else {
    self.current.frequency
};

self.past.frequency * std::cmp::MAX(info.index, 1024) * p_delta

/// MIDI Controller for WereSoCool
///
/// Handles MIDI output via UDP, including note-on/note-off events and MIDI event emission.

use crate::events::Events;
use serde::Serialize;
use std::net::UdpSocket;
use weresocool_instrument::RenderOp;
use weresocool_shared::Settings;

#[derive(Debug)]
pub struct MidiClient {
    socket: UdpSocket,
    addr: String,
}

impl MidiClient {
    pub fn new(addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("127.0.0.1:0")?; // ephemeral port
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            addr: addr.to_string(),
        })
    }

    pub fn send(&self, msg: &impl Serialize) {
        if let Ok(buf) = serde_json::to_vec(msg) {
            if let Err(e) = self.socket.send_to(&buf, &self.addr) {
                eprintln!("weresocool: failed to send MIDI UDP: {}", e);
            }
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum MidiMsg {
    NoteOnAt {
        ch: u8,
        note: u8,
        vel: u8,
        delay_ms: u64,
    },
    NoteOffAt {
        ch: u8,
        note: u8,
        vel: u8,
        delay_ms: u64,
    },
    PanAt {
        ch: u8,
        value: u8,
        delay_ms: u64,
    },
}

pub fn freq_to_midi_note(freq_hz: f64, a4: f64) -> u8 {
    if freq_hz <= 0.0 {
        return 0;
    }
    let midi = 69.0 + 12.0 * (freq_hz / a4).log2();
    midi.round().clamp(0.0, 127.0) as u8
}

/// MIDI Controller manages MIDI output for audio rendering
#[derive(Debug)]
pub struct MidiController {
    client: MidiClient,
}

impl MidiController {
    pub fn new(client: MidiClient) -> Self {
        Self { client }
    }

    /// Send MIDI events for the given render operations
    ///
    /// This fires MIDI note-on at operation start and note-off at operation end,
    /// with timing delays relative to the read start position.
    pub fn send_midi_events(
        &self,
        midi_ops: &[RenderOp],
        read_start_samples: usize,
        events: &mut Events,
    ) {
        let a4 = 440.0f64;
        let sr = Settings::global().sample_rate as f64;

        for op in midi_ops {
            if op.ext.midi.is_empty() {
                continue;
            }

            // Voice→channel mapping (overlay):
            let ch1: u8 = if op.ext.midi.len() == 1 {
                let base = op.ext.midi[0].max(1).min(16);
                (((base - 1) as usize + op.voice) % 16) as u8 + 1
            } else {
                op.ext.midi[op.voice % op.ext.midi.len()]
            };
            let ch = (ch1.saturating_sub(1)).min(15);

            let note = freq_to_midi_note(op.f, a4);
            let pan_val = (((op.p + 1.0) / 2.0) * 127.0)
                .round()
                .clamp(0.0, 127.0) as u8;

            let is_start = op.index == 0;
            let is_end = op.index + op.samples >= op.total_samples;

            // Compute delays relative to this read start, based on op.t (seconds)
            let start_samples = (op.t * sr).round() as usize;
            let end_samples = start_samples.saturating_add(op.total_samples);
            let delay_on_ms = if start_samples > read_start_samples {
                ((start_samples - read_start_samples) as f64 / sr * 1000.0).round() as u64
            } else {
                0
            };
            let delay_off_ms = if end_samples > read_start_samples {
                ((end_samples - read_start_samples) as f64 / sr * 1000.0).round() as u64
            } else {
                0
            };

            if is_start {
                // Use velocity to control per-hit loudness (works best for drums)
                // Map op.gain_scalar in [0.0..2.0] to [1..127], with Gm 1.0 -> 100
                let mut vel_f = (op.gain_scalar * 100.0).round();
                if vel_f < 1.0 {
                    vel_f = 1.0;
                }
                if vel_f > 127.0 {
                    vel_f = 127.0;
                }
                let vel = vel_f as u8;
                self.client.send(&MidiMsg::PanAt {
                    ch,
                    value: pan_val,
                    delay_ms: delay_on_ms,
                });
                self.client.send(&MidiMsg::NoteOnAt {
                    ch,
                    note,
                    vel,
                    delay_ms: delay_on_ms,
                });
            }
            if is_end {
                self.client.send(&MidiMsg::NoteOffAt {
                    ch,
                    note,
                    vel: 64,
                    delay_ms: delay_off_ms,
                });
            }
        }

        // Emit MIDI event if there are subscribers and we have MIDI ops
        if !midi_ops.is_empty() && events.midi.has_subscribers() {
            let timestamp = read_start_samples as f64 / Settings::global().sample_rate;
            events.midi.emit(crate::events::MidiEvent {
                ops: midi_ops.to_vec(),
                timestamp,
            });
        }
    }
}

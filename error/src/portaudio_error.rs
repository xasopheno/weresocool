use failure::Fail;
use portaudio;
use serde::{Deserialize, Serialize};
use std::fmt;

impl fmt::Display for PortAudioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PortAudioError")
    }
}

#[derive(Serialize, Deserialize, Debug, Fail)]
#[serde(remote = "portaudio::error::Error")]
pub enum PortAudioError {
    /// No Error
    NoError = 0,
    /// Portaudio not initialized
    NotInitialized = -10000,
    /// Unanticipated error from the host
    UnanticipatedHostError,
    /// Invalid channel count
    InvalidChannelCount,
    /// Invalid sample rate
    InvalidSampleRate,
    /// Invalid Device
    InvalidDevice,
    /// Invalid Flag
    InvalidFlag,
    /// The Sample format is not supported
    SampleFormatNotSupported,
    /// Input device not compatible with output device
    BadIODeviceCombination,
    /// Memory insufficient
    InsufficientMemory,
    /// The buffer is too big
    BufferTooBig,
    /// The buffer is too small
    BufferTooSmall,
    /// Invalid callback
    NullCallback,
    /// Invalid Stream
    BadStreamPtr,
    /// Time out
    TimedOut,
    /// Portaudio internal error
    InternalError,
    /// Device unavailable
    DeviceUnavailable,
    /// Stream info not compatible with the host
    IncompatibleHostApiSpecificStreamInfo,
    /// The stream is stopped
    StreamIsStopped,
    /// The stream is not stopped
    StreamIsNotStopped,
    /// The input stream has overflowed
    InputOverflowed,
    /// The output has underflowed
    OutputUnderflowed,
    /// The host API is not found by Portaudio
    HostApiNotFound,
    /// The host API is invalid
    InvalidHostApi,
    /// Portaudio cannot read from the callback stream
    CanNotReadFromACallbackStream,
    /// Portaudio cannot write to the callback stream
    CanNotWriteToACallbackStream,
    /// Portaudio cannot read from an output only stream
    CanNotReadFromAnOutputOnlyStream,
    /// Portaudio cannot write to an input only stream
    CanNotWriteToAnInputOnlyStream,
    /// The stream is not compatible with the host API
    IncompatibleStreamHostApi,
    /// Invalid buffer
    BadBufferPtr,
}

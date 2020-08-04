import librosa
import scipy.io.wavfile

sampling_rate = 44100
y, sr = librosa.load("./my_song.wav")

y_shifted = librosa.effects.pitch_shift(
    y, sr, n_steps=-40.0, bins_per_octave=24
)  # shifted by 4 half steps

y_stretch = librosa.effects.time_stretch(y_shifted, rate=0.25)


print(y_shifted)
scipy.io.wavfile.write("test.wav", 44100, y_shifted)
scipy.io.wavfile.write("test_2.wav", 44100, y_stretch)

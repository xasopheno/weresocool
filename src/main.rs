mod yin;
mod sine;
mod set_elements;

fn main() { 
    let sample_rate: f32 = 44_100.0;
    let buffer_size: f32 = 2048.0;
    let threshold = 0.15;
    let freq: f32 = 1100.0;

    println!("generated freq is {}", freq);
    
    let buffer = sine::generate_sinewave(sample_rate, buffer_size, freq);
    
    yin::yin_pitch_detection(buffer, sample_rate, threshold);
}

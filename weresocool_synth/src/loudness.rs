use weresocool_shared::Settings;

pub fn loudness_normalization(frequency: f64) -> f64 {
    let normalization = freq_to_sones(frequency);
    if normalization.is_nan() || normalization.is_infinite() || normalization > 1.0 {
        1.0
    } else {
        normalization
    }
}

fn in_rendered_range(frequency: f64) -> bool {
    let settings = Settings::global();
    frequency >= settings.min_freq
}

fn freq_to_sones(frequency: f64) -> f64 {
    // http://www.ukintpress-conferences.com/conf/08txeu_conf/pdf/day_1/01-06-garcia.pdf
    if in_rendered_range(frequency) {
        1.0 / (((20.0 * (frequency).log10()) - 40.0) / 10.0).exp2()
    } else {
        0.0
    }
}

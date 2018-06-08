fn main() {
    let test_case1 = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0];
    
    let yd = yin_difference(test_case1);
    println!("yin_difference: \n {:?}", yd);
    let cmnd = yin_cumulative_mean_normalized_difference(yd);
    println!("cum_mean_norm_difference \n {:?}", cmnd);
    
    let yat = yin_absolute_threshold(cmnd.clone());
    let tau: usize;
    let probability: f32;
    match yat {
        Some(yat) => {
            println!("absolute_threshold \n {:?}", yat);
            probability = 1.0 - cmnd[yat];
            tau = yat;
            println!("probability \n {}", probability);
            },
        _ => {
            println!("{}", "Did not work"); 
            panic!();
        }
    } 
    let better_tau = yin_parabolic_interpolation(cmnd, tau);
    println!("better tau after parabolic interpolation \n {:?}", better_tau);

    let pitch_in_hertz = 4410.0 / better_tau;
    println!("pitch in hertz {:?}", pitch_in_hertz)
}

fn yin_difference(buffer: Vec<f32>) -> Vec<f32> {
    let mut buffer_clone = buffer.clone();
    
    let half_buffer_size = &buffer_clone.len() / 2;
    
    for tau in 0..half_buffer_size {
        for i in 0..half_buffer_size {
            let delta: f32 = buffer[i] - buffer[i + tau];
 			buffer_clone[tau] += delta * delta;
        };
    };
    buffer_clone
}

fn yin_cumulative_mean_normalized_difference(buffer: Vec<f32>) -> Vec<f32> {
    let mut buffer_clone = buffer.clone();
    
    let half_buffer_size = &buffer_clone.len() / 2;
    let mut running_sum: f32 = 0.0;
    
    for tau in 1..half_buffer_size {
        running_sum += buffer_clone[tau];
        buffer_clone[tau] *= tau as f32 / running_sum;
    };

    buffer_clone
}


fn yin_absolute_threshold(buffer: Vec<f32>) -> Option<usize> {
    let threshold = 0.20;
    let mut iter = buffer
        .iter()
        .enumerate()
        .skip(2)
        .skip_while(|(_, &sample)| sample > threshold);
    let tripped_threshold = iter.next()?;

    let (_, mut previous_sample) = tripped_threshold;
    for (index, sample) in iter {
        if sample > previous_sample {
            return Some(index - 1);
        };
        previous_sample = sample;
    };

    Some(buffer.len() - 1)
}

fn yin_parabolic_interpolation(buffer: Vec<f32>, tau_estimate: usize) -> f32 {
    let better_tau: f32;
    let x0: usize;
    let x2: usize;

	if tau_estimate < 1 {
        x0 = tau_estimate;
    } else {
        x0 = tau_estimate - 1;
    }

    if tau_estimate + 1 < buffer.len() / 2 {
		x2 = tau_estimate + 1;
	} 
	else {
		x2 = tau_estimate;
	}

    if x0 == tau_estimate {
		if buffer[tau_estimate] <= buffer[x2] {
			better_tau = tau_estimate as f32;
		} 
		else {
			better_tau = x2 as f32;
		}
	} 

    else if x2 == tau_estimate {
		if buffer[tau_estimate] <= buffer[x0] {
			better_tau = tau_estimate as f32;
		} 
		else {
			better_tau = x0 as f32;
		}
	} else {
		let s0: f32 = buffer[x0];
		let s1: f32 = buffer[tau_estimate];
		let s2: f32 = buffer[x2];

		better_tau = tau_estimate as f32 + (s2 - s0) / (2.0 * (2.0 * s1 - s2 - s0));
	}

    better_tau
}

#[cfg(test)]
mod tests {
    use yin_difference;
    #[test]
    fn yin_difference_test() {
        let test_case = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];
        // let expected = vec![1.0, 2.2, 3.1];
        
        // assert_eq!(yin_difference(test_case), 
        // expected);
    }
}


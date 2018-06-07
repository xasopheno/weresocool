fn main() {
    let test_case1 = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];

    let test_case2 = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0];
    let yd = yin_difference(test_case1);
    println!("yin_difference: {:?}", yd);
    let cmnd = yin_cumulative_mean_normalized_difference(yd);
    println!("cum_mean_norm_difference {:?}", cmnd);

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
    let threshold = 0.15;
    let mut iter = buffer
        .iter()
        .enumerate()
        .skip(2)
        .skip_while(|(_, &sample)| sample < threshold);
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


// int16_t Yin_absoluteThreshold(Yin *yin){
// 	int16_t tau;

// 	/* Search through the array of cumulative mean values, and look for ones that are over the threshold 
// 	 * The first two positions in yinBuffer are always so start at the third (index 2) */
// 	for (tau = 2; tau < yin->halfBufferSize ; tau++) {
// 		if (yin->yinBuffer[tau] < yin->threshold) {
// 			while (tau + 1 < yin->halfBufferSize && yin->yinBuffer[tau + 1] < yin->yinBuffer[tau]) {
// 				tau++;
// 			}
// 			/* found tau, exit loop and return
// 			 * store the probability
// 			 * From the YIN paper: The yin->threshold determines the list of
// 			 * candidates admitted to the set, and can be interpreted as the
// 			 * proportion of aperiodic power tolerated
// 			 * within a periodic signal.
// 			 *
// 			 * Since we want the periodicity and and not aperiodicity:
// 			 * periodicity = 1 - aperiodicity */
// 			yin->probability = 1 - yin->yinBuffer[tau];
// 			break;
// 		}
// 	}

// 	/* if no pitch found, tau => -1 */
// 	if (tau == yin->halfBufferSize || yin->yinBuffer[tau] >= yin->threshold) {
// 		tau = -1;
// 		yin->probability = 0;
// 	}

// 	return tau;
// }

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


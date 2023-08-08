// MIT License

// Copyright (c) 2023 Ryan Andersen

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use std::time::Instant;

use rand::{distributions::Distribution, rngs::ThreadRng};

use fast_loaded_dice_roller as fldr;

#[derive(Default)]
struct TestCoin {
    pub fair_coin: fldr::rand::RngCoin<ThreadRng>,
    pub flip_count: usize,
}

impl fldr::FairCoin for TestCoin {
    fn flip(&mut self) -> bool {
        self.flip_count += 1;
        self.fair_coin.flip()
    }
}

// TODO: ? Perhaps comparing the speed against the naive implementation is not reasonable.
// Test currently fails in release mode with the series of if-else statements out-performing FLDR for the chosen distribution.
#[ignore]
#[test]
fn test_speed() {
    const ROLL_COUNT: usize = 1_000_000;

    // Store results in a histogram.
    let mut histogram = [0usize; 10];

    // Perform a pre-determined number of rolls and record the results.
    let basic_rng_duration = {
        // Create a new thread-local PRNG.
        let mut rng = ThreadRng::default();

        // Create a uniform distribution to target with the RNG.
        let uniform_distribution = rand::distributions::Uniform::new(0., 1.);

        let basic_rng_start = Instant::now();
        for _ in 0..ROLL_COUNT {
            let s = uniform_distribution.sample(&mut rng);
            if s >= 1. {
                histogram[0] += 1;
            } else if s >= 0.9777777 {
                histogram[1] += 1;
            } else if s >= 0.9333333 {
                histogram[2] += 1;
            } else if s >= 0.8666666 {
                histogram[3] += 1;
            } else if s >= 0.7777777 {
                histogram[4] += 1;
            } else if s >= 0.6666666 {
                histogram[5] += 1;
            } else if s >= 0.5333333 {
                histogram[6] += 1;
            } else if s >= 0.3777777 {
                histogram[7] += 1;
            } else if s >= 0.2 {
                histogram[8] += 1;
            } else if s >= 0. {
                histogram[9] += 1;
            }
        }
        let basic_rng_duration = basic_rng_start.elapsed();

        println!(
            "Uniform sampling: Time: {:?}, Hist: {:?}",
            basic_rng_duration, histogram
        );
        basic_rng_duration
    };

    // Reset the histogram.
    histogram.iter_mut().for_each(|x| *x = 0);

    // Perform a pre-determined number of rolls and record the results.
    let fldr_rng_start = {
        // Create a new fair coin.
        let mut fair_coin = fldr::rand::RngCoin::default();
        let test_distribution = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(test_distribution.len(), histogram.len());

        // Create a discrete-distribution-generator from a list of weights.
        let generator = fldr::Generator::new(&test_distribution);

        let fldr_rng_start = Instant::now();
        for _ in 0..ROLL_COUNT {
            let i = generator.sample(&mut fair_coin);
            histogram[i] += 1;
        }
        let fldr_rng_duration = fldr_rng_start.elapsed();

        println!("FLDR: Time: {:?}, Hist: {:?}", fldr_rng_duration, histogram);
        fldr_rng_duration
    };

    assert!(fldr_rng_start < basic_rng_duration);
}

#[test]
fn test_entropy() {
    const RANDOM_DISTRIBUTION_COUNT: usize = 10;

    // Create a new thread-local PRNG to unbias the testing.
    let mut rng = ThreadRng::default();
    // Use bytes uniformly distributed between 0 and 255.
    let uniform_distribution = rand::distributions::Uniform::new(0, 256);

    // Perform a pre-determined number of rolls and record the results.
    for _ in 0..RANDOM_DISTRIBUTION_COUNT {
        const ROLL_COUNT: usize = 100_000;
        const DISTRIBUTION_SIZE: usize = 10;

        // Create a new fair coin for tracking the number of flips used.
        let mut fair_coin = TestCoin::default();

        // Create a random distribution to target with the RNG.
        let test_distribution = std::iter::repeat_with(|| uniform_distribution.sample(&mut rng))
            .take(DISTRIBUTION_SIZE)
            .collect::<Vec<_>>();
        let test_total: usize = test_distribution.iter().sum();

        // Create a discrete-distribution-generator from a list of weights.
        let generator = fldr::Generator::new(&test_distribution);

        // Perform the rolls.
        for _ in 0..ROLL_COUNT {
            generator.sample(&mut fair_coin);
        }

        let flips_per_sample = fair_coin.flip_count as f64 / ROLL_COUNT as f64;
        let expected_bits_per_sample =
            f64::from(test_total.ilog2()) + f64::from(!test_total.is_power_of_two()) + 6.5;
        assert!(
            flips_per_sample < 8. * DISTRIBUTION_SIZE as f64 + 7.,
            "The number of flips per sample should never be greater (on the average) than this upper bound. Flips per sample: {flips_per_sample}"
        );
        assert!(
            flips_per_sample / expected_bits_per_sample < 1.01,
            "The number of flips per sample should not be 1% greater than the expected number of flips per sample. Expected: {expected_bits_per_sample}, Actual: {flips_per_sample}"
        );
    }
}

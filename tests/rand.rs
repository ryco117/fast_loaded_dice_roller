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
use rand::{distr::Distribution as _, rngs::ThreadRng};

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

#[test]
fn test_entropy() {
    const RANDOM_DISTRIBUTION_COUNT: usize = 10;

    // Create a new thread-local PRNG to unbias the testing.
    let mut rng = ThreadRng::default();
    // Use bytes uniformly distributed between 0 and 255.
    let uniform_distribution =
        rand::distr::Uniform::try_from(0..256).expect("Failed to create uniform distribution.");

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

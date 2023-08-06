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

//! The Fast Loaded Dice Roller (FLDR) algorithm is a space and time efficient algorithm for near-optimal sampling from a weighted discrete distribution.
//! This crate provides an implementation of the FLDR algorithm that is generic over the type of random number generator (RNG) used;
//! there is an optional implementation that uses the `rand` crate as a dependency which can be enabled by using the `rand` feature of this crate.

/// Sampling from the FLDR requires a fair coin, i.e. a random variable that outputs `true` or `false` with equal probability.
/// This trait describes the interface for a fair coin, but lets the user choose the specifics of how to implement it.
pub trait FairCoin {
    fn flip(&mut self) -> bool;
}

/// The discrete-distribution-generator (DDG) tree used to randomly sample items with specified weights.
/// The FLDR algorithm operates on this object to maintain a size that scales linearly with the number
/// of bits needed to encode the input distribution.
pub struct Generator {
    bucket_count: usize,
    level_label_matrix: Vec<Vec<usize>>,
    level_leaf_count: Vec<usize>,
}

impl Generator {
    /// Create a new DDG tree for the FLDR algorithm from a list of non-negative integer weights.
    /// # Panics
    /// Will panic if `distribution` has less than two non-zero weights.
    #[must_use]
    pub fn new(distribution: &[usize]) -> Self {
        assert!(
            distribution.iter().fold(0, |c, &w| c + usize::from(w > 0)) >= 2,
            "The distribution must have at least two non-zero weights."
        );
        let bucket_count = distribution.len();
        let sum: usize = distribution.iter().sum();
        let is_power_of_two = sum.is_power_of_two();

        // Get the ceiling of the base 2 logarithm of `sum`.
        let depth: usize = sum.ilog2() as usize + usize::from(!is_power_of_two);

        let a: Vec<_> = if is_power_of_two {
            // Copy the existing distribution to owned memory.
            distribution.to_vec()
        } else {
            // Append an element to the distribution to make the new sum a power of two.
            (0..=bucket_count)
                .map(|i| {
                    if i < bucket_count {
                        distribution[i]
                    } else {
                        (1 << depth) - sum
                    }
                })
                .collect()
        };

        // Create dynamic arrays for storing the DDG tree in a compact way.
        let mut level_leaf_count = vec![0; depth];
        let mut level_label_matrix: Vec<_> = std::iter::repeat(vec![0; bucket_count + 1])
            .take(depth)
            .collect(); // TODO: Use a sparse matrix representation?

        // Iterate over the levels of the tree.
        for j in 0..depth {
            let mut label_index = 0;
            let level_labels = &mut level_label_matrix[j];

            // Iterate over the labels in the (possibly appended) distribution.
            for (i, &a) in a.iter().enumerate() {
                // Use the binary expansion of the weight for label i to determine the locations of this label in the tree.
                // E.g., if the weight is 6 (110 in binary) and the tree depth is 3, then the label will have a leaf at the first and second level.
                // The intuition is that larger weights will be closer to the root in the tree, and will have more leaves with their label based on their hamming weight.
                if (a >> (depth - j - 1)) & 1 > 0 {
                    level_leaf_count[j] += 1;
                    level_labels[label_index] = i;
                    label_index += 1;
                }
            }
        }

        Self {
            bucket_count,
            level_label_matrix,
            level_leaf_count,
        }
    }

    /// Sample a random item from the discrete distribution using a given `FairCoin`.
    pub fn sample(&self, fair_coin: &mut impl FairCoin) -> usize {
        let mut label_index = 0;
        let mut level = 0;

        // Traverse the binary tree with coin flips until a leaf is reached.
        loop {
            // Flip a fair coin for random sample outputs.
            let toss = fair_coin.flip();

            // Bit shift the index and add the coin toss to choose a child in the tree.
            label_index = (label_index << 1) + usize::from(toss);

            // Check the index is within the current tree level.
            if label_index < self.level_leaf_count[level] {
                // Check the label here is within the actual distribution and is not the appended value.
                if self.level_label_matrix[level][label_index] < self.bucket_count {
                    // Return the sampled label.
                    return self.level_label_matrix[level][label_index];
                }

                // Take a back-edge to the root of the tree/graph.
                label_index = 0;
                level = 0;
            } else {
                // Wrap the label index by the level leaf count.
                label_index -= self.level_leaf_count[level];

                // Increase to the next level in the tree.
                level += 1;
            }
        }
    }
}

#[cfg(feature = "rand")]
pub mod rand {
    use rand::{rngs::ThreadRng, Rng};

    /// Helper type for performing repeated coin flips.
    /// Fetches random bits from a given RNG in blocks of 64 bits and return them one at a time.
    pub struct RngCoin<R: Rng> {
        rng: R,
        random_bits: u64,
        bits_read: u32,
    }

    impl<R: Rng> RngCoin<R> {
        /// Create a new `RngCoin` instance with the given RNG and assign a random `u64` to `random_bits`.
        #[must_use]
        pub fn new(mut rng: R) -> Self {
            let random_bits = rng.next_u64();
            Self {
                rng,
                random_bits,
                bits_read: 0,
            }
        }
    }

    /// Create a new `RngCoin` and default to using the local `ThreadRng` instance RNG.
    impl Default for RngCoin<ThreadRng> {
        fn default() -> Self {
            RngCoin::new(ThreadRng::default())
        }
    }

    /// Implement the `FairCoin` trait so that this struct can be sampled by the FLDR `Generator`.
    impl<R: Rng> super::FairCoin for RngCoin<R> {
        fn flip(&mut self) -> bool {
            // If we have read the entire `u64` of random bits, then we need to generate a new block.
            if self.bits_read == u64::BITS {
                self.random_bits = self.rng.next_u64();
                self.bits_read = 0;
            }

            // Grab the right-most bit and increment the number of bits read.
            let b = self.random_bits & 1 > 0;
            self.bits_read += 1;

            // Shift the random bits to the right by one and return the result bit.
            self.random_bits >>= 1;
            b
        }
    }
}

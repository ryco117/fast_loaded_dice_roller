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

//! # Fast Loaded Dice Roller
//! The [Fast Loaded Dice Roller (FLDR)](https://arxiv.org/pdf/2003.03830.pdf)[\*](#citation)
//! algorithm is a space and time efficient algorithm for near-optimal sampling from a weighted
//! discrete distribution. This crate provides an implementation of the FLDR algorithm that is
//! generic over the type of random number generator (RNG) used. There is an optional
//! implementation, `rand::RngCoin<R>`, that uses the `rand` crate as a dependency for convenience.
//!
//! ### Citation
//! I neither created nor discovered the FLDR algorithm. This crate is simply an implementation.
//!
//! \* Citation for the Fast Loaded Dice Roller algorithm:
//! ```text
//! @inproceedings{saad2020fldr,
//!   title           = {The Fast Loaded Dice Roller: A Near-optimal Exact Sampler for Discrete Probability Distributions},
//!   author          = {Saad, Feras A. and Freer, Cameron E. and Rinard, Martin C. and Mansinghka, Vikash K.},
//!   booktitle       = {AISTATS 2020: Proceedings of the 23rd International Conference on Artificial Intelligence and Statistics},
//!   volume          = 108,
//!   series          = {Proceedings of Machine Learning Research},
//!   address         = {Palermo, Sicily, Italy},
//!   publisher       = {PMLR},
//!   year            = 2020,
//!   keywords        = {random variate generation, sampling, discrete random variables},
//!   abstract        = {This paper introduces a new algorithm for the fundamental problem of generating a random integer from a discrete probability distribution using a source of independent and unbiased random coin flips. This algorithm, which we call the Fast Loaded Dice Roller (FLDR), has efficient complexity properties in space and time: the size of the sampler is guaranteed to be linear in the number of bits needed to encode the target distribution and the sampler consumes (in expectation) at most 6.5 bits of entropy more than the information-theoretically minimal rate, independently of the values or size of the target distribution. We present an easy-to-implement, linear-time preprocessing algorithm and a fast implementation of the FLDR using unsigned integer arithmetic. Empirical evaluations establish that the FLDR is 2x--10x faster than multiple baseline algorithms for exact sampling, including the widely-used alias and interval samplers. It also uses up to 10000x less space than the information-theoretically optimal sampler, at the expense of a less than 1.5x runtime overhead.},
//! }
//! ```

/// Sampling from the FLDR requires a fair coin, i.e. a random variable that outputs `true` or
/// `false` with equal probability. This trait describes the interface for a fair coin, but lets
/// the user choose the specifics of how to implement it.
pub trait FairCoin {
    /// A coin flip takes no inputs and returns one of two values with equal probability.
    /// NOTE: The coin is taken as a mutable reference because implementations will likely need to
    /// update their internal state in order to sample new random numbers.
    fn flip(&mut self) -> bool;
}

/// Represents the discrete-distribution-generator (DDG) tree used to randomly sample items with
/// specified weights. The FLDR algorithm operates on this object to maintain a size that scales
/// linearly with the number of bits needed to encode the input distribution.
#[derive(Clone)]
pub struct Generator {
    bucket_count: usize,
    adjusted_bucket_count: usize,
    level_label_matrix: Vec<usize>,
}

impl Generator {
    /// Create a new DDG tree for the FLDR algorithm from a list of non-negative integer weights.
    /// # Panics
    /// Will panic if `distribution` has less than two non-zero weights.
    #[must_use]
    pub fn new(distribution: &[usize]) -> Self {
        assert!(
            distribution.iter().filter(|&&w| w > 0).count() >= 2,
            "The distribution must have at least two non-zero weights."
        );
        let bucket_count = distribution.len();
        let sum: usize = distribution.iter().sum();
        let is_power_of_two = sum.is_power_of_two();

        // Get the ceiling of the base 2 logarithm of `sum`.
        // This will let us create a binary tree with a depth that is as small as possible while
        // still being able to represent the sum of the weights.
        let depth: usize = sum.ilog2() as usize + usize::from(!is_power_of_two);

        let a: Vec<_> = if is_power_of_two {
            // Copy the existing distribution to owned memory.
            distribution.to_vec()
        } else {
            // Append an element to the distribution to make the new sum a power of two.
            // As we'll see, this is crucial to utilizing unsigned integer arithmetic to build our
            // DDG tree.
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

        // Create a matrix to store the labels that occur within each level of the tree,
        // as well as the number of labels in that level.
        // TODO: Try to store this matrix in a sparse representation to save space.
        // However, data locality is important for performance, so we'll need to be careful.
        let mut level_label_matrix = vec![0; (a.len() + 1) * depth];

        // Iterate over the levels of the DDG tree and populate them with the appropriate entries.
        for j in 0..depth {
            // Iterate over the labels in the (possibly appended) distribution.
            for (i, &w) in a.iter().enumerate() {
                // Use the binary expansion of the weight for label `i` to determine the locations
                // of this label in the tree. The sum has been adjusted to be a power of two, so
                // when dividing each weight by the total to get the probability, the action
                // amounts to a bit shift. Thus, the probability is encoded in the binary expansion
                // of the weight.
                //
                // E.g., if the tree depth is 4 and the weight is 5 (0101 in binary), then the
                // label will have a leaf at the second and fourth level. Ignoring back-edges, this
                // represents a probability of 1/4 (second level) + 1/16 (fourth level) = 5/16.
                // This makes sense given a weight of 5 and a sum upper bounded by 16 (which we
                // know since the example depth is 4). The intuition is that larger weights will be
                // closer to the root in the tree, thus more likely to be sampled, and will have
                // more leaves assigned their label based on their hamming weight.
                if (w >> (depth - j - 1)) & 1 > 0 {
                    // Use `k` to index into the start of the level in the matrix.
                    let k = j * (a.len() + 1);

                    // Increase the number of labels in the current level.
                    let count = {
                        level_label_matrix[k] += 1;
                        level_label_matrix[k]
                    };

                    // Add the label to the current level.
                    level_label_matrix[k + count] = i;
                }
            }
        }

        Self {
            bucket_count,
            adjusted_bucket_count: a.len(),
            level_label_matrix,
        }
    }

    /// Sample a random item from the discrete distribution using a given `FairCoin`.
    /// The item is returned as an index into the initial input distribution.
    pub fn sample(&self, fair_coin: &mut impl FairCoin) -> usize {
        let mut label_index = 0;
        let mut level = 0;

        // Traverse the binary tree with coin flips until a leaf is reached.
        loop {
            // Flip a fair coin for random sample outputs.
            let toss = fair_coin.flip();

            // Bit shift the index and add the coin toss to choose a random child in the tree.
            label_index = (label_index << 1) + usize::from(toss);

            // Use `k` to index into the start of the level in the matrix.
            let k = level * (self.adjusted_bucket_count + 1);

            // Check the index is within the current tree level.
            if label_index < self.level_label_matrix[k] {
                // Check the label here is within the actual distribution and is not the appended value.
                let j = self.level_label_matrix[k + label_index + 1];
                if j < self.bucket_count {
                    // Return the sampled label.
                    return j;
                }

                // Take a back-edge to the root of the tree/graph.
                label_index = 0;
                level = 0;
            } else {
                // Wrap the label index by the level's leaf count.
                label_index -= self.level_label_matrix[k];

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

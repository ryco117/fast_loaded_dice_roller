/// Sampling from the Fast Loaded Dice Roller requires a fair coin, i.e. a random variable that outputs `true` or `false` with equal probability.
/// This trait describes the interface for a fair coin, but lets the user implement it however they want.
pub trait FairCoin {
    fn flip(&mut self) -> bool;
}

/// The discrete-distribution-generator tree used to randomly sample items with specified weights.
/// The Fast Loaded Dice Roller algorithm operates on this object to maintain a size O(n)
/// with the number of bits needed to encode the input distribution.
pub struct Generator {
    bucket_count: usize,
    level_label_matrix: Vec<Vec<usize>>,
    level_leaf_count: Vec<usize>,
}

impl Generator {
    /// Create a new generator for the Fast Loaded Dice Roller algorithm from a discrete distribution of non-negative integer weights.
    #[must_use]
    pub fn new(distribution: &[usize]) -> Self {
        let bucket_count = distribution.len();
        let sum: usize = distribution.iter().sum();
        let is_power_of_two = sum.is_power_of_two();

        let depth: usize = sum.ilog2() as usize + usize::from(!is_power_of_two);
        let a: Vec<_> = if is_power_of_two {
            // Copy the distribution to owned memory.
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

        let mut level_leaf_count = vec![0; depth];
        let mut level_label_matrix: Vec<_> = std::iter::repeat(vec![0; bucket_count + 1])
            .take(depth)
            .collect(); // TODO: Use a sparse matrix representation?

        for j in 0..depth {
            let mut label_index = 0;
            let level_labels = &mut level_label_matrix[j];

            // Iterate over the labels in the (possibly appended) distribution.
            for (i, &a) in a.iter().enumerate() {
                // Use the binary expansion of the weight for label i to determine the locations of this label in the tree.
                // E.g., if the weight is 6 (110 in binary) and the depth is 3, then the label will be a leaf at the first level and the second level
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

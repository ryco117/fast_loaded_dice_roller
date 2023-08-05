use rand::RngCore;

const DEFAULT_DISTRIBUTION: [usize; 5] = [0, 1, 2, 3, 4];
const DEFAULT_ROLL_COUNT: usize = 100_000;

// Helper type for performing repeated coin flips.
struct Rng {
    rng: rand::rngs::ThreadRng,
    random_bits: u64,
    bits_read: u32,
}

// Default to using the `ThreadRng` and assign initial random bits.
impl Default for Rng {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let random_bits = rng.next_u64();
        Self {
            rng,
            random_bits,
            bits_read: 0,
        }
    }
}

// Implement the `FairCoin` trait in order to sample from the `fast_loaded_dice_roller` crate.
impl fast_loaded_dice_roller::FairCoin for Rng {
    fn flip(&mut self) -> bool {
        // If we have read the entire `u64` of random bits, then we need to generate a new one.
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

fn main() {
    // Setup simple PRNG for coin flips.
    let mut rng = Rng::default();

    // Setup parameters of the test sampling.
    let distribution = DEFAULT_DISTRIBUTION.to_vec();
    let mut histogram = distribution.iter().map(|_| 0usize).collect::<Vec<_>>();
    let roll_count = DEFAULT_ROLL_COUNT;
    let verbose = false;
    let print_histogram = true;

    // Let 'er roll!
    let roller = fast_loaded_dice_roller::Generator::new(&distribution);
    for _ in 0..roll_count {
        let s = roller.sample(&mut rng);
        if verbose {
            println!("{s}");
        }
        histogram[s] += 1;
    }

    // Print the results of the repeated sampling as a histogram.
    if print_histogram {
        println!(
            "Total rolls: {roll_count}\nInitial distribution: {:?}\nHistogram results: {:?}",
            distribution, histogram
        );
    }
}

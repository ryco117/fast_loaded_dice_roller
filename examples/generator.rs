use rand::RngCore;

const DEFAULT_DISTRIBUTION: [usize; 5] = [0, 1, 2, 3, 4];
const DEFAULT_ROLL_COUNT: usize = 100_000;

// Helper type for performing repeated coin flips.
struct Rng {
    rng: rand::rngs::ThreadRng,
    random_bits: u64,
    bits_read: u32,
}

// Default to using the thread RNG and set random starting bits.
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

// Implement the FairCoin trait to use fldr crate.
impl fldr::FairCoin for Rng {
    fn flip(&mut self) -> bool {
        if self.bits_read == u64::BITS {
            self.random_bits = self.rng.next_u64();
            self.bits_read = 0;
        }

        self.bits_read += 1;
        let b = self.random_bits & 1 > 0;
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
    let roller = fldr::Generator::new(&distribution);
    for _ in 0..roll_count {
        let s = roller.sample(&mut rng);
        if verbose {
            println!("{s}");
        }
        histogram[s] += 1;
    }

    // Print results of the sampling as a histogram.
    if print_histogram {
        println!(
            "Total rolls: {roll_count}\nInitial distribution: {:?}\nHistogram results: {:?}",
            distribution, histogram
        );
    }
}

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

use clap::Parser;

const DEFAULT_DISTRIBUTION: [usize; 5] = [0, 1, 2, 3, 4];
const DEFAULT_ROLL_COUNT: usize = 100_000;

// Use macro and crate `clap` to parse command line arguments.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// The number of independent samples to take from distribution.
    #[arg(short, long, default_value_t = DEFAULT_ROLL_COUNT)]
    roll_count: usize,

    /// Print the results of each sample to their own line.
    #[arg(short, long)]
    verbose: bool,

    /// Silence default behavior to print the total results after all sampling.
    #[arg(short, long)]
    silence_histogram: bool,

    /// The distribution to sample from.
    /// Must have at least two non-zero weights.
    #[arg(short, long, value_parser, num_args = 2..)]
    distribution: Option<Vec<usize>>,
}

fn main() {
    // Parse command line arguments.
    let args = Arguments::parse();

    // Setup simple PRNG for coin flips.
    let mut rng = fast_loaded_dice_roller::rand::RngCoin::default();

    // Setup parameters of the test sampling.
    let distribution = if let Some(dist) = args.distribution {
        dist
    } else {
        DEFAULT_DISTRIBUTION.to_vec()
    };
    let mut histogram = distribution.iter().map(|_| 0usize).collect::<Vec<_>>();
    let roll_count = args.roll_count;
    let verbose = args.verbose;
    let print_histogram = !args.silence_histogram;

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

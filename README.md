# fast_loaded_dice_roller

## About the project

This is a Rust library and example program meant to help popularize the usage of the novel [Fast Loaded Dice Roller](https://arxiv.org/pdf/2003.03830.pdf)[\*](#citation)
discrete sampling algorithm. It is designed with generality, low dependencies, and efficiency in mind. Also, it is simple to use, featuring an optional
default implementation of the required `FairCoin` trait, and heavy documentation of the internal FLDR algorithm for the curious.

## Usage

### Library
The library can be added to your existing projects with `cargo add fast_loaded_dice_roller`.
You can include the optional template `rand::RngCoin<R>` implementation of the `FairCoin` trait by enabling the `rand` feature
(e.g., `cargo add fast_loaded_dice_roller --features="rand"`), which has a dependency on the crate [rand](https://crates.io/crates/rand).

### Example program
The example program can be built with `cargo b --example generator --features="rand"`.
The example can be run with default values, `cargo r --example generator --features="rand"`, and has the following usage:
```
Rust implementation of the novel Fast Loaded Dice Roller algorithm (https://arxiv.org/pdf/2003.03830.pdf)

Usage: generator [OPTIONS]

Options:
  -r, --roll-count <ROLL_COUNT>
          The number of independent samples to take from distribution [default: 100000]
  -v, --verbose
          Print the results of each sample to their own line
  -s, --silence-histogram
          Silence default behavior to print the total results after all sampling
  -d, --distribution <DISTRIBUTION> <DISTRIBUTION>...
          The distribution to sample from. Must have at least two non-zero weights. If not provided, the default distribution is [0, 1, 2, 3, 4]
  -h, --help
          Print help
  -V, --version
          Print version
```
An example of its usage is `cargo r --example generator --features="rand" -- -d 1 2 3 -r 6000`.

## Citation
I neither created nor discovered the FLDR algorithm. This crate is simply an implementation.

\* Citation for the Fast Loaded Dice Roller algorithm:
```
@inproceedings{saad2020fldr,
title           = {The Fast Loaded Dice Roller: A Near-optimal Exact Sampler for Discrete Probability Distributions},
author          = {Saad, Feras A. and Freer, Cameron E. and Rinard, Martin C. and Mansinghka, Vikash K.},
booktitle       = {AISTATS 2020: Proceedings of the 23rd International Conference on Artificial Intelligence and Statistics},
volume          = 108,
series          = {Proceedings of Machine Learning Research},
address         = {Palermo, Sicily, Italy},
publisher       = {PMLR},
year            = 2020,
keywords        = {random variate generation, sampling, discrete random variables},
abstract        = {This paper introduces a new algorithm for the fundamental problem of generating a random integer from a discrete probability distribution using a source of independent and unbiased random coin flips. This algorithm, which we call the Fast Loaded Dice Roller (FLDR), has efficient complexity properties in space and time: the size of the sampler is guaranteed to be linear in the number of bits needed to encode the target distribution and the sampler consumes (in expectation) at most 6.5 bits of entropy more than the information-theoretically minimal rate, independently of the values or size of the target distribution. We present an easy-to-implement, linear-time preprocessing algorithm and a fast implementation of the FLDR using unsigned integer arithmetic. Empirical evaluations establish that the FLDR is 2x--10x faster than multiple baseline algorithms for exact sampling, including the widely-used alias and interval samplers. It also uses up to 10000x less space than the information-theoretically optimal sampler, at the expense of a less than 1.5x runtime overhead.},
}
```

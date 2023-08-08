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

use fast_loaded_dice_roller as fldr;

#[test]
#[should_panic(expected = "The distribution must have at least two non-zero weights.")]
fn test_empty_distribution() {
    // Test a zero-length array.
    let test_distribution = [];
    let _generator = fldr::Generator::new(&test_distribution);
}

#[test]
#[should_panic(expected = "The distribution must have at least two non-zero weights.")]
fn test_unit_distribution() {
    // Test a distribution that contains a single element.
    let test_distribution = [1];
    let _generator = fldr::Generator::new(&test_distribution);
}

#[test]
#[should_panic(expected = "The distribution must have at least two non-zero weights.")]
fn test_all_zero_distribution() {
    // Test a distribution that contains only zeros.
    let test_distribution = [0; 4];
    let _generator = fldr::Generator::new(&test_distribution);
}

#[test]
#[should_panic(expected = "The distribution must have at least two non-zero weights.")]
fn test_lone_weight_distribution() {
    // Test a distribution that contains multiple elements, but only a single non-zero element.
    let test_distribution = [0, 2, 0, 0];
    let _generator = fldr::Generator::new(&test_distribution);
}

use num_bigint::BigUint;
use num_complex::{Complex, Complex64};

#[must_use]
pub fn format_state_id(id: &BigUint, qubit_count: usize) -> String {
    format!("|{}⟩", fmt_basis_state_label(id, qubit_count))
}

#[must_use]
pub fn get_phase(c: &Complex<f64>) -> f64 {
    f64::atan2(c.im, c.re)
}

#[must_use]
pub fn fmt_complex(c: &Complex<f64>) -> String {
    // Format -0 as 0
    // Also using Unicode Minus Sign instead of ASCII Hyphen-Minus
    // and Unicode Mathematical Italic Small I instead of ASCII i.
    format!(
        "{}{:.4}{}{:.4}𝑖",
        if c.re <= -0.00005 { "−" } else { "" },
        c.re.abs(),
        if c.im <= -0.00005 { "−" } else { "+" },
        c.im.abs()
    )
}

#[must_use]
pub fn fmt_basis_state_label(id: &BigUint, qubit_count: usize) -> String {
    // This will generate a bit string that shows the qubits in the order
    // of allocation, left to right.
    format!("{:0>qubit_count$}", id.to_str_radix(2))
}

#[must_use]
pub fn get_latex_for_state2(_state: Vec<(BigUint, Complex64)>, _qubit_count: usize) -> String {
    "$|\\psi\\rangle = TODO 123$".into()
}

#[must_use]
fn is_significant(x: f64) -> bool {
    x.abs() <= 1e-9
}

#[must_use]
fn is_fractional_part_significant(x: f64) -> bool {
    is_significant(x - x.round())
}

// Takes float number
// Returns recognized/not, numerator, denominator
// Sign is in the numerator
fn recognize_nice_rational(x: f64) -> (bool, i64, i64) {
    for denominator in 1..31 {
        // TODO: Optimize this. Consider squares of roots in the denominator.
        let numerator: f64 = x * (denominator as f64);
        if !is_fractional_part_significant(numerator) {
            return (true, numerator.round() as i64, denominator);
        }
    }
    (false, 0, 1)
}

const ROOTS: [i64; 19] = [
    1, 2, 3, 5, 6, 7, 10, 11, 13, 14, 15, 17, 19, 21, 22, 23, 26, 29, 30,
];
fn recognize_nice_algebraic(x: f64, needs_1: bool) -> (bool, String) {
    let is_positive: bool = x >= 0.0;
    let positive_x: f64 = if is_positive { x } else { -x };
    for root in ROOTS {
        // TODO: Optimize this. In practice we don't really need anything beyond 1, √2, √3, √5, √6.
        let by_root: f64 = positive_x / (root as f64).sqrt();
        let (is_rational, numerator, denominator) = recognize_nice_rational(by_root);
        if is_rational {
            return (
                is_positive,
                get_latex_for_algebraic(numerator, denominator, root, needs_1),
            );
        }
    }
    (is_positive, format!("{positive_x}"))
}

// Takes real and imaginary part of a complex number
// returns recognized/not, magnitude_num, magnitude_den, pi_num, pi_den
// Always returns positive magnitude
// Return angles in [-1, 1] * pi, sign is in the pi_num
const PI_FRACTIONS: [(i64, i64); 8] = [
    (1, 3),
    (2, 3),
    (1, 4),
    (3, 4),
    (1, 8),
    (3, 8),
    (5, 8),
    (7, 8),
];
fn recognize_nice_exponent(re: f64, im: f64) -> (bool, i64, i64, i64, i64) {
    for (pi_num, pi_den) in PI_FRACTIONS {
        let angle: f64 = std::f64::consts::PI * (pi_num as f64) / (pi_den as f64);
        let s: f64 = angle.sin();
        let c: f64 = angle.cos();
        if !is_significant(re / c - im / s) {
            // It's OK to take abs value as we are only interested in magnitude
            let (success, magnitude_num, magnitude_den) = recognize_nice_rational((re / c).abs());
            if success {
                if im >= 0.0 {
                    return (true, magnitude_num, magnitude_den, pi_num, pi_den);
                }
                return (true, magnitude_num, magnitude_den, pi_num - pi_den, pi_den);
            }
        }
    }
    (false, 0, 1, 0, 1)
}

fn get_latex_for_algebraic(numerator: i64, denominator: i64, root: i64, needs_1: bool) -> String {
    let actually_needs_1: bool = if denominator != 1 { true } else { needs_1 };
    let mut latex: String = "".into();
    if root == 1 {
        if numerator == 1 {
            if actually_needs_1 {
                latex = "1".into();
            }
        } else {
            latex = format!("{numerator}");
        }
    } else {
        if numerator == 1 {
            latex = format!("\\sqrt{{{root}}}");
        } else {
            latex = format!("{numerator} \\sqrt{{{root}}}");
        }
    }
    if denominator != 1 {
        latex = format!("\\frac{{{latex}}}{{{denominator}}}");
    }

    latex
}

fn get_latex_for_exponent(pi_num: i64, pi_den: i64) -> String {
    let mut latex: String = "e^{{".into();
    let mut abs_pi_num: i64 = pi_num;
    if pi_num < 0 {
        abs_pi_num = -pi_num;
        latex += "-";
    }
    if abs_pi_num != 1 {
        latex += "{abs_pi_num}";
    }
    latex += " i \\pi ";
    if pi_den != 1 {
        latex += " / {pi_den}";
    }
    latex += "}}";

    latex
}

#[must_use]
pub fn get_latex_for_state(state: Vec<(BigUint, Complex64)>, _qubit_count: usize) -> String {
    let mut state_latex: String = "".into();
    let mut term_number: i64 = 0;
    for (basis, amplitude) in state {
        let real: f64 = amplitude.re;
        let imag: f64 = amplitude.im;
        let real_significant: bool = is_significant(real);
        let imag_significant: bool = is_significant(imag);
        if !real_significant && !imag_significant {
            // State is normalized so there's at least one term so we can just skip all 0s.
            continue;
        }
        term_number += 1;

        if real_significant {
            if imag_significant {
                // both real and imaginary amplitude
                let (is_exponent, num, den, pi_num, pi_den) = recognize_nice_exponent(real, imag);
                if is_exponent {
                    if term_number > 1 {
                        state_latex += "+";
                    }
                    state_latex += &get_latex_for_algebraic(num, den, 1, false);
                    state_latex += &get_latex_for_exponent(pi_num, pi_den);
                } else {
                    let (is_positive1, latex1) = recognize_nice_algebraic(real, true);
                    let (is_positive2, latex2) = recognize_nice_algebraic(imag, false);
                    if !is_positive1 {
                        state_latex += "-";
                    } else if term_number > 1 {
                        state_latex += "+";
                    }
                    let imag_sign = if is_positive1 == is_positive2 {
                        "+"
                    } else {
                        "-"
                    };
                    state_latex += &format!("\\left( {latex1} {imag_sign} {latex2}i \\right)");
                }
            } else {
                // only real amplitude
                let (is_positive, latex) = recognize_nice_algebraic(real, false);
                if !is_positive {
                    state_latex += "-";
                } else if term_number > 1 {
                    state_latex += "+";
                }
                state_latex += &format!("{latex} \\ ");
            }
        } else {
            // only imaginary amplitude
            let (is_positive, latex) = recognize_nice_algebraic(imag, false);
            if !is_positive {
                state_latex += "-";
            } else if term_number > 1 {
                state_latex += "+";
            }
            state_latex += &format!("{latex}i \\ ");
        }
        state_latex += &format!("|{basis}\\rangle");
    }

    format!("$|\\psi\\rangle = {state_latex}$")
}

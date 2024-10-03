// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use num_bigint::BigUint;
use num_complex::{Complex, Complex64};
use std::{f64::consts::FRAC_1_SQRT_2, fmt::Write};

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
fn is_significant(x: f64) -> bool {
    x.abs() > 1e-9
}

#[must_use]
fn is_fractional_part_significant(x: f64) -> bool {
    is_significant(x - x.round())
}

/// Represents a non-zero rational number in the form ``numerator/denominator``
/// Sign of the number is separated for easier composition and rendering
#[derive(Copy, Clone, Debug)]
struct RationalNumber {
    sign: i64,        // 1 if the number is positive, -1 if negative
    numerator: i64,   // Positive numerator
    denominator: i64, // Positive denominator
}

impl RationalNumber {
    fn new(numerator: i64, denominator: i64) -> Self {
        Self {
            sign: numerator.signum() * denominator.signum(),
            numerator: numerator.abs(),
            denominator: denominator.abs(),
        }
    }

    fn abs(&self) -> Self {
        Self {
            sign: self.sign.abs(),
            numerator: self.numerator,
            denominator: self.denominator,
        }
    }

    const DENOMINATORS: [i64; 36] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 15, 16, 18, 20, 21, 24, 25, 27, 28, 30, 32, 35, 36,
        40, 42, 45, 48, 49, 50, 54, 56, 60, 63, 64,
    ];

    /// Try to recognize a float number as a "nice" rational.
    fn recognize(x: f64) -> Option<Self> {
        for denominator in Self::DENOMINATORS {
            #[allow(clippy::cast_precision_loss)] // We only use fixes set of denominators
            let numerator: f64 = x * (denominator as f64);
            if numerator.abs() <= 100.0 && !is_fractional_part_significant(numerator) {
                #[allow(clippy::cast_possible_truncation)] // We only allow small numerators
                let rounded_numerator: i64 = numerator.round() as i64;
                return Some(Self::new(rounded_numerator, denominator));
            }
        }
        None
    }
}

/// Represents a non-zero algebraic number in the form ``fraction·√root``,
/// Sign of the number is separated for easier composition and rendering
#[derive(Debug)]
struct AlgebraicNumber {
    sign: i64,                // 1 if the number is positive, -1 if negative
    fraction: RationalNumber, // Positive rational number numerator/denominator
    root: i64,                // Component under square root
}

impl AlgebraicNumber {
    fn new(fraction: &RationalNumber, root: i64) -> Self {
        Self {
            sign: fraction.sign,
            fraction: fraction.abs(),
            root,
        }
    }

    const ROOTS: [i64; 6] = [1, 2, 3, 5, 6, 7];

    /// Try to recognize a float number as a "nice" algebraic.
    fn recognize(x: f64) -> Option<Self> {
        for root in Self::ROOTS {
            #[allow(clippy::cast_precision_loss)] // We only use fixes set of roots
            let divided_by_root: f64 = x / (root as f64).sqrt();
            if let Some(fraction) = RationalNumber::recognize(divided_by_root) {
                return Some(Self::new(&fraction, root));
            }
        }
        None
    }
}

/// Represents a non-zero decimal number as an ``f64`` floating point value.
/// Sign of the number is separated for easier composition and rendering
#[derive(Debug)]
struct DecimalNumber {
    sign: i64,  // 1 if the number is positive, -1 if negative
    value: f64, // Positive floating point value
}

impl DecimalNumber {
    fn new(value: f64) -> Self {
        Self {
            sign: if value >= 0.0 { 1 } else { -1 },
            value: value.abs(),
        }
    }

    // Tries to recognize a decimal number and always succeeds.
    fn recognize(x: f64) -> Self {
        Self::new(x)
    }
}

/// Represents a real number, which can be algebraic, decimal or zero.
#[derive(Debug)]
enum RealNumber {
    Algebraic(AlgebraicNumber),
    Decimal(DecimalNumber),
    Zero,
}

impl RealNumber {
    fn sign(&self) -> i64 {
        match self {
            Self::Algebraic(algebraic) => algebraic.sign,
            Self::Decimal(decimal) => decimal.sign,
            Self::Zero => 0,
        }
    }

    fn negate(&self) -> Self {
        match self {
            Self::Algebraic(algebraic) => Self::Algebraic(AlgebraicNumber {
                sign: -algebraic.sign,
                fraction: algebraic.fraction,
                root: algebraic.root,
            }),
            Self::Decimal(decimal) => Self::Decimal(DecimalNumber {
                sign: -decimal.sign,
                value: decimal.value,
            }),
            Self::Zero => Self::Zero,
        }
    }

    fn abs(&self) -> Self {
        match self {
            Self::Algebraic(algebraic) => Self::Algebraic(AlgebraicNumber {
                sign: 1,
                fraction: algebraic.fraction,
                root: algebraic.root,
            }),
            Self::Decimal(decimal) => Self::Decimal(DecimalNumber {
                sign: 1,
                value: decimal.value,
            }),
            Self::Zero => Self::Zero,
        }
    }

    /// Try to recognize a real number as zero, algebraic, or decimal if all else fails.
    fn recognize(x: f64) -> Self {
        if !is_significant(x) {
            Self::Zero
        } else if let Some(algebraic_number) = AlgebraicNumber::recognize(x) {
            Self::Algebraic(algebraic_number)
        } else {
            Self::Decimal(DecimalNumber::recognize(x))
        }
    }
}

/// Represents a non-zero complex numbers in the polar form: ``magnitude·𝒆^(𝝅·𝒊·phase_multiplier)``
/// Sign of the number is separated for easier composition and rendering
#[derive(Debug)]
struct PolarForm {
    sign: i64,                        // For this form the sign is always 1
    magnitude: AlgebraicNumber,       // magnitude of the number
    phase_multiplier: RationalNumber, // to be multiplied by 𝝅·𝒊 to get phase
}

impl PolarForm {
    fn new(magnitude: &RationalNumber, pi_num: i64, pi_den: i64) -> Self {
        Self {
            sign: 1,
            magnitude: AlgebraicNumber::new(magnitude, 1),
            phase_multiplier: RationalNumber::new(pi_num, pi_den),
        }
    }

    const PI_FRACTIONS: [(i64, i64); 16] = [
        (1, 3),
        (2, 3),
        (1, 4),
        (3, 4),
        (1, 8),
        (3, 8),
        (5, 8),
        (7, 8),
        (1, 16),
        (3, 16),
        (5, 16),
        (7, 16),
        (9, 16),
        (11, 16),
        (13, 16),
        (15, 16),
    ];

    /// Try to recognize a complex number and represent it in the polar form.
    fn recognize(re: f64, im: f64) -> Option<Self> {
        if !is_significant(re.abs()) && !is_significant(im.abs()) {
            // 0 is better represented in Cartesian form, not polar.
            return None;
        }
        for (pi_num, pi_den) in Self::PI_FRACTIONS {
            #[allow(clippy::cast_precision_loss)] // We only use fixes set of fractions
            let angle: f64 = std::f64::consts::PI * (pi_num as f64) / (pi_den as f64);
            let sin: f64 = angle.sin();
            let cos: f64 = angle.cos();
            if is_significant(re / cos - im / sin) {
                continue;
            }
            // We recognized the angle. Now try to recognize magnitude.
            // It's OK to take abs value as we are only interested in magnitude
            if let Some(magnitude) = RationalNumber::recognize((re / cos).abs()) {
                return Some(Self::new(
                    &magnitude,
                    if im >= 0.0 { pi_num } else { pi_num - pi_den },
                    pi_den,
                ));
            }
        }
        None
    }
}

/// Represents a non-zero complex number in the Cartesian form: ``real_part+𝒊·imaginary_part``
/// Sign of the number is separated for easier composition and rendering
#[derive(Debug)]
struct CartesianForm {
    sign: i64,             // 1 the common sign is a "+", -1 is "-", 0 means the number is 0
    real_part: RealNumber, // Real part
    imaginary_part: RealNumber, // Imaginary part
}

impl CartesianForm {
    fn new(real_part: RealNumber, imaginary_part: RealNumber) -> Self {
        let sign = real_part.sign();
        match sign {
            0 => Self {
                sign: imaginary_part.sign(),
                real_part: RealNumber::Zero,
                imaginary_part: imaginary_part.abs(),
            },
            1.. => Self {
                sign,
                real_part,
                imaginary_part,
            },
            _ => Self {
                sign,
                real_part: real_part.abs(),
                imaginary_part: imaginary_part.negate(),
            },
        }
    }

    /// Try to recognize a complex number and represent it in the Cartesian form.
    fn recognize(re: f64, im: f64) -> Self {
        Self::new(RealNumber::recognize(re), RealNumber::recognize(im))
    }
}

/// Represents a non-zero complex number which can be in either polar or Cartesian form.
#[derive(Debug)]
enum ComplexNumber {
    Polar(PolarForm),
    Cartesian(CartesianForm),
}

impl ComplexNumber {
    fn recognize(re: f64, im: f64) -> Self {
        if let Some(exponent) = PolarForm::recognize(re, im) {
            Self::Polar(exponent)
        } else {
            Self::Cartesian(CartesianForm::recognize(re, im))
        }
    }
}

/// Represents one term of a quantum state, which corresponds to one basis vector.
struct Term {
    basis_vector: BigUint,
    coordinate: ComplexNumber,
}

fn get_terms_for_state(state: &Vec<(BigUint, Complex64)>) -> Vec<Term> {
    let mut result: Vec<Term> = Vec::with_capacity(state.len());
    for (basis_vector, coefficient) in state {
        result.push(Term {
            basis_vector: basis_vector.clone(),
            coordinate: ComplexNumber::recognize(coefficient.re, coefficient.im),
        });
    }
    result
}

/// Get the state represented as a formula in the LaTeX format if possible.
/// `None` is returned if the resulting formula is not nice, i.e.
/// if the formula consists of more than 16 terms or if more than two coefficients are not recognized.
#[must_use]
pub fn get_state_latex(state: &Vec<(BigUint, Complex64)>, qubit_count: usize) -> Option<String> {
    if state.len() > 16 {
        return None;
    }

    let terms: Vec<Term> = get_terms_for_state(state);
    let mut bad_term_count: i64 = 0;
    for term in &terms {
        if let ComplexNumber::Cartesian(cartesian) = &term.coordinate {
            if let RealNumber::Decimal(_) = &cartesian.real_part {
                bad_term_count += 1;
            }
            if let RealNumber::Decimal(_) = &cartesian.imaginary_part {
                bad_term_count += 1;
            };
            if bad_term_count > 2 {
                return None;
            }
        }
    }

    let mut latex: String = String::with_capacity(200);
    latex.push_str("$|\\psi\\rangle = ");
    let mut is_first: bool = true;
    for term in terms {
        write_latex_for_term(&mut latex, &term, !is_first);
        let basis_label = fmt_basis_state_label(&term.basis_vector, qubit_count);
        write!(latex, "|{basis_label}\\rangle").expect("Expected to write basis label.");
        is_first = false;
    }
    latex.push('$');
    latex.shrink_to_fit();

    Some(latex)
}

fn is_close_enough(val: &Complex64, target: &Complex64) -> bool {
    (val.re - target.re).abs() < 1e-9 && (val.im - target.im).abs() < 1e-9
}

// Quick and dirty matching for the most common matrix elements we care about rendering
// LaTeX for, e.g.  1/sqrt(2), -i/sqrt(2), etc.
// Anything not in this list gets a standard rendering.
fn get_latex_for_simple_term(val: &Complex64) -> Option<String> {
    if is_close_enough(val, &Complex64::new(FRAC_1_SQRT_2, 0.0)) {
        return Some("\\frac{1}{\\sqrt{2}}".to_string());
    }
    if is_close_enough(val, &Complex64::new(0.0, FRAC_1_SQRT_2)) {
        return Some("\\frac{i}{\\sqrt{2}}".to_string());
    }
    if is_close_enough(val, &Complex64::new(-FRAC_1_SQRT_2, 0.0)) {
        return Some("-\\frac{1}{\\sqrt{2}}".to_string());
    }
    if is_close_enough(val, &Complex64::new(0.0, -FRAC_1_SQRT_2)) {
        return Some("-\\frac{i}{\\sqrt{2}}".to_string());
    }
    if is_close_enough(val, &Complex64::new(0.0, 0.5)) {
        return Some("\\frac{i}{2}".to_string());
    }
    if is_close_enough(val, &Complex64::new(0.0, -0.5)) {
        return Some("-\\frac{i}{2}".to_string());
    }
    if is_close_enough(val, &Complex64::new(0.5, 0.5)) {
        return Some("\\frac{1}{2} + \\frac{i}{2}".to_string());
    }
    if is_close_enough(val, &Complex64::new(-0.5, -0.5)) {
        return Some("-\\frac{1}{2} - \\frac{i}{2}".to_string());
    }
    if is_close_enough(val, &Complex64::new(-0.5, 0.5)) {
        return Some("-\\frac{1}{2} + \\frac{i}{2}".to_string());
    }
    if is_close_enough(val, &Complex64::new(0.5, -0.5)) {
        return Some("\\frac{1}{2} - \\frac{i}{2}".to_string());
    }
    None
}

#[must_use]
pub fn get_matrix_latex(matrix: &Vec<Vec<Complex64>>) -> String {
    let mut latex: String = String::with_capacity(500);
    latex.push_str("$ \\begin{bmatrix} ");
    for row in matrix {
        let mut is_first: bool = true;
        for element in row {
            if !is_first {
                latex.push_str(" & ");
            }
            is_first = false;

            if let Some(simple_latex) = get_latex_for_simple_term(element) {
                latex.push_str(&simple_latex);
                continue;
            }

            let cpl = ComplexNumber::recognize(element.re, element.im);
            write_latex_for_complex_number(&mut latex, &cpl);
        }
        latex.push_str(" \\\\ ");
    }
    latex.push_str("\\end{bmatrix} $");
    latex.shrink_to_fit();
    latex
}

/// Write latex for a standalone complex number
/// '-', 0 and 1 are always rendered, but '+' is not.
fn write_latex_for_complex_number(latex: &mut String, number: &ComplexNumber) {
    match number {
        ComplexNumber::Cartesian(cartesian_form) => {
            write_latex_for_cartesian_form(latex, cartesian_form, false, true);
        }
        ComplexNumber::Polar(polar_form) => {
            write_latex_for_polar_form(latex, polar_form, false);
        }
    }
}

/// Write latex for one term of quantum state.
/// Latex is rendered for coefficient only (not for basis vector).
/// + is rendered only if ``render_plus`` is true.
fn write_latex_for_term(latex: &mut String, term: &Term, render_plus: bool) {
    match &term.coordinate {
        ComplexNumber::Cartesian(cartesian_form) => {
            write_latex_for_cartesian_form(latex, cartesian_form, render_plus, false);
        }
        ComplexNumber::Polar(polar_form) => {
            write_latex_for_polar_form(latex, polar_form, render_plus);
        }
    }
}

/// Write latex for polar form of a complex number.
/// The sign is always + and is rendered only if ``render_plus`` is true.
fn write_latex_for_polar_form(latex: &mut String, polar_form: &PolarForm, render_plus: bool) {
    if polar_form.sign < 0 {
        latex.push('-');
    } else if render_plus {
        latex.push('+');
    }
    write_latex_for_algebraic_number(latex, &polar_form.magnitude, false);
    latex.push_str(" e^{");
    if polar_form.phase_multiplier.sign < 0 {
        latex.push('-');
    }
    let pi_num: i64 = polar_form.phase_multiplier.numerator;
    if pi_num != 1 {
        write!(latex, "{pi_num}").expect("Expected to write phase numerator.");
    }
    latex.push_str(" i \\pi");
    let pi_den = polar_form.phase_multiplier.denominator;
    if pi_den != 1 {
        write!(latex, " / {pi_den}").expect("expected to write phase denominator.");
    }
    latex.push('}');
}

/// Write latex for cartesian form of a complex number.
/// Common + is rendered only if ``render_plus`` is true.
/// Brackets are used if both real and imaginary parts are present.
/// If only one part is present, its sign is used as common.
/// If both components are present, real part sign is used as common.
/// 1 is rendered if ``render_one`` is true
/// + is rendered if ``render_plus`` is true.
fn write_latex_for_cartesian_form(
    latex: &mut String,
    cartesian_form: &CartesianForm,
    render_plus: bool,
    render_one: bool,
) {
    if cartesian_form.sign < 0 {
        latex.push('-');
    } else if render_plus {
        latex.push('+');
    }
    if let RealNumber::Zero = cartesian_form.real_part {
        if let RealNumber::Zero = cartesian_form.imaginary_part {
            latex.push('0');
        } else {
            // Only imaginary part present
            write_latex_for_real_number(latex, &cartesian_form.imaginary_part, false);
            latex.push('i');
        }
    } else if let RealNumber::Zero = cartesian_form.imaginary_part {
        // Only real part present
        write_latex_for_real_number(latex, &cartesian_form.real_part, render_one);
    } else {
        // Both real and imaginary parts present
        latex.push_str("\\left( ");
        write_latex_for_real_number(latex, &cartesian_form.real_part, true);
        latex.push(if cartesian_form.imaginary_part.sign() < 0 {
            '-'
        } else {
            '+'
        });
        write_latex_for_real_number(latex, &cartesian_form.imaginary_part, false);
        latex.push_str("i \\right)");
    }
}

/// Write latex for real number. Note that the sign is not rendered.
/// 1 is only rendered if ``render_one`` is true.
fn write_latex_for_real_number(latex: &mut String, number: &RealNumber, render_one: bool) {
    match number {
        RealNumber::Algebraic(algebraic_number) => {
            write_latex_for_algebraic_number(latex, algebraic_number, render_one);
        }
        RealNumber::Decimal(decimal_number) => {
            write_latex_for_decimal_number(latex, decimal_number, render_one);
        }
        RealNumber::Zero => {
            latex.push('0');
        }
    }
}

/// Write latex for decimal number. Note that the sign is not rendered.
/// 1 is only rendered if ``render_one`` is true.
fn write_latex_for_decimal_number(latex: &mut String, number: &DecimalNumber, render_one: bool) {
    if render_one || is_significant(number.value - 1.0) {
        // Using round() instead of neater string formatting({:.4})
        // because we do not want trailing zeros (we need 0.5 and not 0.5000)
        write!(latex, "{}", (number.value * 10000.0).round() / 10000.0)
            .expect("Expected to write decimal value.");
    }
}

/// Write latex for algebraic number. Note that the sign is not rendered.
/// 1 is only rendered if ``render_one`` is true.
fn write_latex_for_algebraic_number(
    latex: &mut String,
    number: &AlgebraicNumber,
    render_one: bool,
) {
    let actually_needs_1: bool = render_one || number.fraction.denominator != 1;
    if number.fraction.denominator != 1 {
        latex.push_str("\\frac{");
    }
    if number.root == 1 {
        if number.fraction.numerator == 1 {
            if actually_needs_1 {
                latex.push('1');
            }
        } else {
            write!(latex, "{}", number.fraction.numerator).expect("Expected to write numerator.");
        }
    } else if number.fraction.numerator == 1 {
        write!(latex, "\\sqrt{{{}}}", number.root)
            .expect("Expected to write square root expression.");
    } else {
        write!(
            latex,
            "{} \\sqrt{{{}}}",
            number.fraction.numerator, number.root
        )
        .expect("expected to write numerator and square root expression.");
    }
    if number.fraction.denominator != 1 {
        write!(latex, "}}{{{}}}", number.fraction.denominator)
            .expect("Expected to write denominator.");
    }
}

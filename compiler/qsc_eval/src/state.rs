#[cfg(test)]
mod tests;

use num_bigint::BigUint;
use num_complex::{Complex, Complex64};
use std::fmt::Write;

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

// Represents a non-zero rational number in the form numerator/denominator
// Sign of the number is separated for easier composition and rendering
#[derive(Copy, Clone)]
struct RationalNumber {
    sign: i64,        // 1 if the number is positive, -1 if negative
    numerator: i64,   // Positive numerator
    denominator: i64, // Positive denominator
}

impl RationalNumber {
    fn construct(numerator: i64, denominator: i64) -> Self {
        Self {
            sign: numerator.signum(),
            numerator: numerator.abs(),
            denominator,
        }
    }

    fn abs(self) -> RationalNumber {
        RationalNumber {
            sign: 1,
            numerator: self.numerator,
            denominator: self.denominator,
        }
    }

    // Tries to recognize a float number as rational.
    fn recognize(x: f64) -> Option<Self> {
        // TODO: Optimize this. Consider squares of roots in the denominator.
        for denominator in 1..31 {
            let numerator: f64 = x * (denominator as f64);
            if !is_fractional_part_significant(numerator) {
                let rounded_numerator: i64 = numerator.round() as i64;
                return Some(Self::construct(rounded_numerator, denominator));
            }
        }
        None
    }
}

// Represents a non-zero algebraic number in the form fraction·√root,
// Sign of the number is separated for easier composition and rendering
#[derive(Copy, Clone)]
struct AlgebraicNumber {
    sign: i64,                // 1 if the number is positive, -1 if negative
    fraction: RationalNumber, // Positive rational number numerator/denominator
    root: i64,                // square root component
}

impl AlgebraicNumber {
    fn construct(fraction: RationalNumber, root: i64) -> Self {
        Self {
            sign: fraction.sign,
            fraction: fraction.abs(),
            root,
        }
    }

    const ROOTS: [i64; 19] = [
        1, 2, 3, 5, 6, 7, 10, 11, 13, 14, 15, 17, 19, 21, 22, 23, 26, 29, 30,
    ];

    // Tries to recognize a float number as algebraic.
    fn recognize(x: f64) -> Option<Self> {
        // TODO: Optimize this. In practice we don't really need anything beyond 1, √2, √3, √5, √6.
        for root in Self::ROOTS {
            let divided_by_root: f64 = x / (root as f64).sqrt();
            if let Some(fraction) = RationalNumber::recognize(divided_by_root) {
                return Some(Self::construct(fraction, root));
            }
        }
        None
    }
}

// Represents a non-zero decimal number as an f64 floating point value.
// Sign of the number is separated for easier composition and rendering
#[derive(Copy, Clone)]
struct DecimalNumber {
    sign: i64,  // 1 if the number is positive, -1 if negative
    value: f64, // Positive floating point value
}

impl DecimalNumber {
    fn construct(value: f64) -> Self {
        Self {
            sign: if value >= 0.0 { 1 } else { -1 },
            value: value.abs(),
        }
    }

    // Tries to recognize a decimal number and always succeeds.
    fn recognize(x: f64) -> Self {
        Self::construct(x)
    }
}

// Represents a real number, which can be algebraic, decimal or zero.
#[derive(Copy, Clone)]
enum RealNumber {
    Algebraic(AlgebraicNumber),
    Decimal(DecimalNumber),
    Zero,
}

impl RealNumber {
    fn sign(self) -> i64 {
        match self {
            RealNumber::Algebraic(algebraic) => algebraic.sign,
            RealNumber::Decimal(decimal) => decimal.sign,
            RealNumber::Zero => 0,
        }
    }
    fn negate(self) -> RealNumber {
        match self {
            RealNumber::Algebraic(algebraic) => RealNumber::Algebraic(AlgebraicNumber {
                sign: -algebraic.sign,
                fraction: algebraic.fraction,
                root: algebraic.root,
            }),
            RealNumber::Decimal(decimal) => RealNumber::Decimal(DecimalNumber {
                sign: -decimal.sign,
                value: decimal.value,
            }),
            RealNumber::Zero => self,
        }
    }
    fn abs(self) -> RealNumber {
        match self {
            RealNumber::Algebraic(algebraic) => RealNumber::Algebraic(AlgebraicNumber {
                sign: 1,
                fraction: algebraic.fraction,
                root: algebraic.root,
            }),
            RealNumber::Decimal(decimal) => RealNumber::Decimal(DecimalNumber {
                sign: 1,
                value: decimal.value,
            }),
            RealNumber::Zero => self,
        }
    }

    // Tries to recognize a real number as zero, algebraic, or decimal of all else fails.
    fn recognize(x: f64) -> RealNumber {
        if !is_significant(x) {
            RealNumber::Zero
        } else if let Some(algebraic_number) = AlgebraicNumber::recognize(x) {
            RealNumber::Algebraic(algebraic_number)
        } else {
            RealNumber::Decimal(DecimalNumber::recognize(x))
        }
    }
}

// Represents a non-zero complex numbers in the polar form: coefficient·𝒆^(𝝅·𝒊·phase_multiplier)
// Sign of the number is separated for easier composition and rendering
struct PolarForm {
    sign: i64, // 1 means the number should be rendered with "+", -1 is "-", 0 means no sign
    magnitude: AlgebraicNumber, // magnitude of the number
    phase_multiplier: RationalNumber, // to be multiplied by 𝝅·𝒊 to get phase
}

impl PolarForm {
    fn construct(magnitude: RationalNumber, pi_num: i64, pi_den: i64) -> Self {
        Self {
            sign: 1,
            magnitude: AlgebraicNumber::construct(magnitude, 1),
            phase_multiplier: RationalNumber::construct(pi_num, pi_den),
        }
    }

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

    fn recognize(re: f64, im: f64) -> Option<Self> {
        for (pi_num, pi_den) in Self::PI_FRACTIONS {
            let angle: f64 = std::f64::consts::PI * (pi_num as f64) / (pi_den as f64);
            let sin: f64 = angle.sin();
            let cos: f64 = angle.cos();
            if is_significant(re / cos - im / sin) {
                continue;
            }
            // We recognized the angle. Now try to recognize magnitude.
            // It's OK to take abs value as we are only interested in magnitude
            if let Some(magnitude) = RationalNumber::recognize((re / cos).abs()) {
                return Some(Self::construct(
                    magnitude,
                    if im >= 0.0 { pi_num } else { pi_num - pi_den },
                    pi_den,
                ));
            }
        }
        None
    }
}

// Represents a non-zero complex number in the Cartesian form: real_part+𝒊·imaginary_part
// Sign of the number is separated for easier composition and rendering
struct CartesianForm {
    sign: i64, // 1 means the number should be rendered with "+", -1 is "-", 0 means no sign
    real_part: RealNumber, // Real part
    imaginary_part: RealNumber, // Imaginary part
}

impl CartesianForm {
    fn construct(real_part: RealNumber, imaginary_part: RealNumber) -> Self {
        let sign = real_part.sign();
        if sign == 0 {
            CartesianForm {
                sign: imaginary_part.sign(),
                real_part: real_part.negate(), // TODO: check...
                imaginary_part: imaginary_part.abs(),
            }
        } else if sign < 0 {
            CartesianForm {
                sign,
                real_part: real_part.abs(),
                imaginary_part: imaginary_part.negate(),
            }
        } else {
            CartesianForm {
                sign,
                real_part,
                imaginary_part,
            }
        }
    }
    fn recognize(re: f64, im: f64) -> CartesianForm {
        CartesianForm::construct(RealNumber::recognize(re), RealNumber::recognize(im))
    }
}

// Represents a non-zero complex number which can be in either Polar or CartesianForm
enum ComplexNumber {
    Polar(PolarForm),
    Cartesian(CartesianForm),
}

impl ComplexNumber {
    fn recognize(re: f64, im: f64) -> ComplexNumber {
        if let Some(exponent) = PolarForm::recognize(re, im) {
            ComplexNumber::Polar(exponent)
        } else {
            ComplexNumber::Cartesian(CartesianForm::recognize(re, im))
        }
    }
}

// Represents one term of a quantum state which corresponds to one basis vector
struct Term {
    basis_vector: BigUint, // TODO: See if it's better to borrow
    coordinate: ComplexNumber,
}

fn get_terms_for_state(state: Vec<(BigUint, Complex64)>) -> Vec<Term> {
    let mut result: Vec<Term> = Vec::with_capacity(state.len());
    for (basis_vector, coefficient) in state {
        // TODO: Better to drop insignificant coordinates.
        result.push(Term {
            basis_vector,
            coordinate: ComplexNumber::recognize(coefficient.re, coefficient.im),
        });
    }
    result
}

// =========================
#[must_use]
pub fn get_latex_for_state(state: Vec<(BigUint, Complex64)>, qubit_count: usize) -> String {
    let terms: Vec<Term> = get_terms_for_state(state);
    let mut latex: String = String::with_capacity(200);

    latex.push_str("$|\\psi\\rangle = ");
    let mut is_first: bool = true;
    for term in terms {
        write_latex_for_term(&mut latex, &term, !is_first);
        let basis_label = fmt_basis_state_label(&term.basis_vector, qubit_count);
        write!(latex, "|{basis_label}\\rangle").unwrap();
        is_first = false;
    }
    latex.push('$');
    latex.shrink_to_fit();

    latex
}

fn write_latex_for_term(latex: &mut String, term: &Term, render_plus: bool) {
    match &term.coordinate {
        ComplexNumber::Cartesian(cartesian_form) => {
            write_latex_for_cartesian_form(latex, cartesian_form, render_plus);
        }
        ComplexNumber::Polar(polar_form) => {
            write_latex_for_polar_form(latex, polar_form, render_plus);
        }
    }
}

fn write_latex_for_polar_form(latex: &mut String, complex_number: &PolarForm, render_plus: bool) {
    if complex_number.sign < 0 {
        latex.push('-');
    } else if render_plus {
        latex.push('+');
    }
    write_latex_for_algebraic_number(latex, complex_number.magnitude, false);
    latex.push_str(" e^{");
    if complex_number.phase_multiplier.sign < 0 {
        latex.push('-');
    }
    let pi_num: i64 = complex_number.phase_multiplier.numerator;
    if pi_num != 1 {
        write!(latex, "{pi_num}").unwrap();
    }
    latex.push_str(" i \\pi ");
    let pi_den = complex_number.phase_multiplier.denominator;
    if pi_den != 1 {
        write!(latex, " / {pi_den}").unwrap();
    }
    latex.push('}');
}

fn write_latex_for_cartesian_form(
    latex: &mut String,
    cartesian_form: &CartesianForm,
    render_plus: bool,
) {
    if cartesian_form.sign < 0 {
        latex.push('-');
    } else if render_plus {
        latex.push('+');
    }
    if let RealNumber::Zero = cartesian_form.real_part {
        if let RealNumber::Zero = cartesian_form.imaginary_part {
            // TODO: This is an empty coefficient. Shouldn't happen.
        } else {
            // Only imaginary part present
            write_latex_for_real_number(latex, cartesian_form.imaginary_part, false);
            latex.push('i');
        }
    } else {
        if let RealNumber::Zero = cartesian_form.imaginary_part {
            // Only real part present
            write_latex_for_real_number(latex, cartesian_form.real_part, false);
        } else {
            // Both real and imaginary parts present
            latex.push_str("\\left( ");
            write_latex_for_real_number(latex, cartesian_form.real_part, true);
            latex.push(if cartesian_form.imaginary_part.sign() < 0 {
                '-'
            } else {
                '+'
            });
            write_latex_for_real_number(latex, cartesian_form.imaginary_part, false);
            latex.push_str("i \\right)");
        }
    }
}

fn write_latex_for_real_number(latex: &mut String, number: RealNumber, render_one: bool) {
    match number {
        RealNumber::Algebraic(algebraic_number) => {
            write_latex_for_algebraic_number(latex, algebraic_number, render_one);
        }
        RealNumber::Decimal(decimal_number) => {
            write_latex_for_decimal_number(latex, decimal_number, render_one);
        }
        RealNumber::Zero => {
            // Note: this arm is not used.
            latex.push('0');
        }
    }
}

fn write_latex_for_decimal_number(latex: &mut String, number: DecimalNumber, render_one: bool) {
    if render_one || is_significant(number.value - 1.0) {
        write!(latex, "{}", number.value).unwrap();
    }
}

fn write_latex_for_algebraic_number(latex: &mut String, number: AlgebraicNumber, render_one: bool) {
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
            write!(latex, "{}", number.fraction.numerator).unwrap();
        }
    } else {
        if number.fraction.numerator == 1 {
            write!(latex, "\\sqrt{{{}}}", number.root).unwrap();
        } else {
            write!(
                latex,
                "{} \\sqrt{{{}}}",
                number.fraction.numerator, number.root
            )
            .unwrap();
        }
    }
    if number.fraction.denominator != 1 {
        write!(latex, "}}{{{}}}", number.fraction.denominator).unwrap();
    }
}

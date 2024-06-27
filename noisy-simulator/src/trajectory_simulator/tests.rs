use num_complex::Complex;
use crate::{instrument::Instrument, operation::{Operation, operation}, TOLERANCE};
use super::TrajectorySimulator;

macro_rules! assert_approx_eq {
    ($left:expr, $right:expr $(,)?) => {
        if !approx_eq($left, $right) {
            panic!("aprox_equal failed, left = {}, right = {}", $left, $right);
        }
    };
}

fn approx_eq (a: f64, b: f64) -> bool {
    (a-b).abs() <= TOLERANCE
}

fn h() -> Operation {
    let f = 0.5_f64.sqrt();
    operation!([f,  f;
                f, -f;])
}

fn cnot() -> Operation {
    operation!([1., 0., 0., 0.;
                0., 1., 0., 0.;
                0., 0., 0., 1.;
                0., 0., 1., 0.;])
}

fn mz() -> Instrument {
    let m0 = operation!([1., 0.;
                         0., 0.;]);
    let m1 = operation!([0., 0.;
                         0., 1.;]);
    Instrument::new(vec![m0, m1])
}

#[test]
fn constructor() {
    let _sim = TrajectorySimulator::new(1);
}

#[test]
fn one_qubit() {
    let h = h();
    let mz = mz();

    for _ in 0..10 {
        let mut sim = TrajectorySimulator::new(1);
        sim.apply_operation(&h, &[0]);
        let measurement = sim.sample_instrument_with_distribution(&mz, &[0], 0.3);
        assert_eq!(measurement, 0);
        println!("--");
    }

    println!(":: :: ::");

    for _ in 0..10 {
        let mut sim = TrajectorySimulator::new(1);
        println!("OOOOOOOOOOOOOOOOOOOOOOOO");
        sim.apply_operation(&h, &[0]);
        println!("HHHHHHHHHHHHHHHHHHHHHHHH");
        let measurement = sim.sample_instrument_with_distribution(&mz, &[0], 0.7);
        assert_eq!(measurement, 1);
    }
}

#[test]
fn bell_pair_sampling() {
    let (h, cnot, mz) = (h(), cnot(), mz());    

    for _ in 0..10 {
        let mut sim = TrajectorySimulator::new(2);
        sim.apply_operation(&h, &[0]);
        sim.apply_operation(&cnot, &[1, 0]);
        let m1 = sim.sample_instrument(&mz, &[0]);
        let m2 = sim.sample_instrument(&mz, &[1]);
        assert_eq!(m1, m2);
    }
}

fn bell_pair_projection(outcome: usize) {
    assert!((0..4).contains(&outcome));
    let (h, cnot, mz) = (h(), cnot(), mz());
    let mut sim = TrajectorySimulator::new(2);
    sim.apply_operation(&h, &[0]);
    sim.apply_operation(&cnot, &[1, 0]);
    sim.apply_operation(mz.operation(outcome & 1), &[0]);

    if outcome == 0 || outcome == 3 {
        sim.apply_operation(mz.operation((outcome >> 1) & 1), &[1]);
        assert_approx_eq!(0.5, sim.trace_change());
        sim.apply_operation(mz.operation(outcome & 1), &[0]);
        sim.apply_operation(mz.operation((outcome >> 1) & 1), &[1]);
        assert_approx_eq!(0.5, sim.trace_change());
    } else {
        sim.apply_operation(mz.operation((outcome >> 1) & 1), &[1]);
    }
}

#[test]
fn bell_pair_projection_pass() {
    bell_pair_projection(0);
    bell_pair_projection(3);
}

#[test]
#[should_panic(expected = "numerical error; failed to sample Kraus operators")]
fn bell_pair_projection_panic_1() {
    bell_pair_projection(1);
}

#[test]
#[should_panic(expected = "numerical error; failed to sample Kraus operators")]
fn bell_pair_projection_panic_2() {
    bell_pair_projection(2);
}


fn two_qubit_gate(outcome: usize) {
    assert!((0..4).contains(&outcome));
    let h = h();
    let m0 = operation!([1., 0.;
                         0., 0.;]);
    let m1 = operation!([0., 0.;
                         0., 1.;]);
    let mz = mz();
    let probabilities: Vec<f64> = vec![0.05, 0.1, 0.3, 0.7, 0.8, 0.9, 0.99];    
    let crx = |t: f64| {
        let c = t.cos();
        let s = t.sin() * Complex::I;
        operation!(
            [1., 0., 0., 0.;
             0., 1., 0., 0.;
             0., 0., c,  s;
             0., 0., s,  c;
            ]
        )
    };

    for _ in 0..10 {
        for p in &probabilities {
            let t = p.sqrt().acos();
            let b1 = (outcome & 1) != 0;
            let b2 = (outcome >> 1) != 0;

            let mut sim = TrajectorySimulator::new(2);
            sim.apply_operation(&h, &[0]);
            sim.apply_operation(&crx(0.3 * t), &[1,0]);
            sim.apply_operation(&crx(0.7 * t), &[1,0]);
            sim.apply_operation(if b1 { &m1 } else { &m0 }, &[0]);
            
            if b1 {
                assert_approx_eq!(0.5, sim.trace_change());
                sim.apply_operation(if b2 { &m1 } else { &m0 }, &[1]);
                assert_approx_eq!(0.5 * if b2 { 1. - p } else { *p }, sim.trace_change());
                sim.apply_operation(if b2 { &m1 } else { &m0 }, &[1]);
                assert_approx_eq!(0.5 * if b2 { 1. - p } else { *p }, sim.trace_change());
            } else {
                assert_eq!(0, sim.sample_instrument(&mz, &[1]));
                assert_approx_eq!(0.5, sim.trace_change());
                sim.apply_operation(&m1, &[1]);
            }
        }
    }
}

#[test]
fn two_qubit_gate_pass() {
    two_qubit_gate(1);
    two_qubit_gate(3);
}

#[test]
#[should_panic(expected = "numerical error; failed to sample Kraus operators")]
fn two_qubit_gate_panic_0() {
    two_qubit_gate(0);
}

#[test]
#[should_panic(expected = "numerical error; failed to sample Kraus operators")]
fn two_qubit_gate_panic_2() {
    two_qubit_gate(2);
}

#[test]
fn repeated_mz() {
    let h = h();
    let mz = mz();
    let mut sim = TrajectorySimulator::new(1);

    for _ in 0..20 {
        sim.apply_operation(&h, &[0]);
        let outcome_0 = sim.sample_instrument(&mz, &[0]);
        let outcome_1 = sim.sample_instrument(&mz, &[0]);
        assert_eq!(outcome_0, outcome_1);
    }
}

#[test]
fn alternating_mz_and_mx() {
    let h = h();
    let mz = mz();
    let mx = Instrument::new(vec![
        operation!([
            0.5, 0.5;
            0.5, 0.5;
        ]),
        operation!([
             0.5, -0.5;
            -0.5,  0.5;
        ])
    ]);

    let mut sim = TrajectorySimulator::new(1);
    sim.apply_operation(&h, &[0]);
    let mut prob = 1.0;
    
    for _ in 0..5 {
        sim.sample_instrument(&mz, &[0]);
        prob *= 0.5;
        assert_approx_eq!(prob, sim.trace_change());
        sim.sample_instrument(&mx, &[0]);
        prob *= 0.5;
        assert_approx_eq!(prob, sim.trace_change());
    }
}
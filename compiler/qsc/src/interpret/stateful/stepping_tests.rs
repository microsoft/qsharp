// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::interpret::stateful::Interpreter;
use qsc_eval::{output::CursorReceiver, StepAction, StepResult};
use qsc_fir::fir::StmtId;
use qsc_frontend::compile::{SourceMap, TargetProfile};
use qsc_passes::PackageType;
use std::io::Cursor;

#[cfg(test)]
mod given_interpreter_with_sources {
    use super::*;

    static STEPPING_SOURCE: &str = r#"
        namespace Test {
            @EntryPoint()
            operation A() : Int {
                let d = B();
                let e = d / 1;
                e
            }
            operation B() : Int {
                let g = 10;
                let h = 20;
                let l = C(g, h);
                42
            }
            operation C(m: Int, n: Int) : Int {
                let o = 42 - (m + n);
                let p = (m + n) + o;
                p
            }
        }"#;
    #[cfg(test)]
    mod step {
        use super::*;

        #[test]
        fn in_one_level_operation_works() -> Result<(), Vec<crate::interpret::stateful::Error>> {
            let sources = SourceMap::new([("test".into(), STEPPING_SOURCE.into())], None);
            let mut interpreter =
                Interpreter::new(true, sources, PackageType::Exe, TargetProfile::Full)?;
            interpreter.set_entry()?;
            let ids = get_breakpoint_ids(&interpreter, "test");
            let expected_id = ids[0];
            expect_bp(&mut interpreter, &ids, expected_id);
            expect_in(&mut interpreter);
            expect_next(&mut interpreter);
            expect_next(&mut interpreter);
            expect_next(&mut interpreter);
            expect_next(&mut interpreter);
            expect_next(&mut interpreter);
            let expected = "42";
            expect_return(interpreter, expected);
            Ok(())
        }

        #[test]
        fn next_crosses_operation_works() -> Result<(), Vec<crate::interpret::stateful::Error>> {
            let sources = SourceMap::new([("test".into(), STEPPING_SOURCE.into())], None);
            let mut interpreter =
                Interpreter::new(true, sources, PackageType::Exe, TargetProfile::Full)?;
            interpreter.set_entry()?;
            let ids = get_breakpoint_ids(&interpreter, "test");
            let expected_id = ids[0];
            expect_bp(&mut interpreter, &ids, expected_id);
            expect_next(&mut interpreter);
            expect_next(&mut interpreter);
            let expected = "42";
            expect_return(interpreter, expected);
            Ok(())
        }

        #[test]
        fn in_multiple_operations_works() -> Result<(), Vec<crate::interpret::stateful::Error>> {
            let sources = SourceMap::new([("test".into(), STEPPING_SOURCE.into())], None);
            let mut interpreter =
                Interpreter::new(true, sources, PackageType::Exe, TargetProfile::Full)?;
            interpreter.set_entry()?;
            let ids = get_breakpoint_ids(&interpreter, "test");
            let expected_id = ids[0];
            expect_bp(&mut interpreter, &ids, expected_id);
            expect_in(&mut interpreter);
            expect_next(&mut interpreter);
            expect_next(&mut interpreter);
            expect_in(&mut interpreter);
            expect_next(&mut interpreter);
            expect_next(&mut interpreter);
            expect_next(&mut interpreter);
            expect_next(&mut interpreter);
            expect_next(&mut interpreter);
            let expected = "42";
            expect_return(interpreter, expected);
            Ok(())
        }

        #[test]
        fn out_multiple_operations_works() -> Result<(), Vec<crate::interpret::stateful::Error>> {
            let sources = SourceMap::new([("test".into(), STEPPING_SOURCE.into())], None);
            let mut interpreter =
                Interpreter::new(true, sources, PackageType::Exe, TargetProfile::Full)?;
            interpreter.set_entry()?;
            let ids = get_breakpoint_ids(&interpreter, "test");
            let expected_id = ids[0];
            expect_bp(&mut interpreter, &ids, expected_id);
            expect_in(&mut interpreter);
            expect_next(&mut interpreter);
            expect_next(&mut interpreter);
            expect_in(&mut interpreter);
            expect_out(&mut interpreter);
            expect_out(&mut interpreter);
            expect_next(&mut interpreter);
            let expected = "42";
            expect_return(interpreter, expected);
            Ok(())
        }
    }
}

fn get_breakpoint_ids(interpreter: &Interpreter, path: &str) -> Vec<StmtId> {
    let mut bps = interpreter.get_breakpoints(path);
    bps.sort_by_key(|f| f.id);
    let ids = bps.iter().map(|f| f.id.into()).collect::<Vec<_>>();
    ids
}

fn expect_return(mut interpreter: Interpreter, expected: &str) {
    let r = step_next(&mut interpreter, &[]);
    match r.0 {
        Ok(StepResult::Return(value)) => assert_eq!(value.to_string(), expected),
        Ok(v) => panic!("Expected Return, got {v:?}"),
        Err(e) => panic!("Expected Return, got {e:?}"),
    }
}

fn expect_bp(interpreter: &mut Interpreter, ids: &[StmtId], expected_id: StmtId) {
    let r = step_next(interpreter, ids);
    match r.0 {
        Ok(StepResult::BreakpointHit(actual_id)) => assert!(actual_id == expected_id),
        Ok(v) => panic!("Expected BP, got {v:?}"),
        Err(e) => panic!("Expected BP, got {e:?}"),
    }
}

fn step_in(
    interpreter: &mut Interpreter,
    breakpoints: &[StmtId],
) -> (
    Result<StepResult, Vec<crate::interpret::stateful::Error>>,
    String,
) {
    step(interpreter, breakpoints, qsc_eval::StepAction::In)
}

fn step_next(
    interpreter: &mut Interpreter,
    breakpoints: &[StmtId],
) -> (
    Result<StepResult, Vec<crate::interpret::stateful::Error>>,
    String,
) {
    step(interpreter, breakpoints, qsc_eval::StepAction::Next)
}

fn step_out(
    interpreter: &mut Interpreter,
    breakpoints: &[StmtId],
) -> (
    Result<StepResult, Vec<crate::interpret::stateful::Error>>,
    String,
) {
    step(interpreter, breakpoints, qsc_eval::StepAction::Out)
}

fn step(
    interpreter: &mut Interpreter,
    breakpoints: &[StmtId],
    step: StepAction,
) -> (
    Result<StepResult, Vec<crate::interpret::stateful::Error>>,
    String,
) {
    let mut cursor = Cursor::new(Vec::<u8>::new());
    let mut receiver = CursorReceiver::new(&mut cursor);
    (
        interpreter.eval_step(&mut receiver, breakpoints, step),
        receiver.dump(),
    )
}

fn expect_next(interpreter: &mut Interpreter) {
    let result = step_next(interpreter, &[]);
    match result.0 {
        Ok(StepResult::Next) => (),
        Ok(v) => panic!("Expected Next, got {v:?}"),
        Err(e) => panic!("Expected Next, got {e:?}"),
    }
}

fn expect_in(interpreter: &mut Interpreter) {
    let result = step_in(interpreter, &[]);
    match result.0 {
        Ok(StepResult::StepIn) => (),
        Ok(v) => panic!("Expected StepIn, got {v:?}"),
        Err(e) => panic!("Expected StepIn, got {e:?}"),
    }
}

fn expect_out(interpreter: &mut Interpreter) {
    let result = step_out(interpreter, &[]);
    match result.0 {
        Ok(StepResult::StepOut) => (),
        Ok(v) => panic!("Expected StepOut, got {v:?}"),
        Err(e) => panic!("Expected StepOut, got {e:?}"),
    }
}

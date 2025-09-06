// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{assert_block_instructions, assert_blocks, assert_callable, get_rir_program};
use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::{BlockId, CallableId};

#[test]
fn unitary_call_within_a_for_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                for _ in 1..3 {
                    op(q);
                }
            }
        }
        "#,
    });

    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: op
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[163-167] scope=1
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=1
                Variable(0, Integer) = Store Integer(2) !dbg package_id=2 span=[163-167] scope=2 discriminator=1
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=2
                Variable(0, Integer) = Store Integer(3) !dbg package_id=2 span=[163-167] scope=2 discriminator=2
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=3
                Variable(0, Integer) = Store Integer(4) !dbg package_id=2 span=[163-167] scope=2 discriminator=3
                Call id(2), args( Integer(0), Pointer, ) !dbg package_id=2 span=[105-109]
                Return !dbg package_id=2 span=[105-109]"#]],
    );
}

#[test]
fn unitary_call_within_a_while_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable idx = 0;
                while idx < 3 {
                    op(q);
                    set idx += 1;
                }
            }
        }
        "#,
    });

    let rotation_callable_id = CallableId(1);
    assert_callable(
        &program,
        rotation_callable_id,
        &expect![[r#"
        Callable:
            name: op
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[162-165] scope=0
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[0-0] scope=1 discriminator=1
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[230-233] scope=1 discriminator=1
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[0-0] scope=1 discriminator=2
                Variable(0, Integer) = Store Integer(2) !dbg package_id=2 span=[230-233] scope=1 discriminator=2
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[0-0] scope=1 discriminator=3
                Variable(0, Integer) = Store Integer(3) !dbg package_id=2 span=[230-233] scope=1 discriminator=3
                Call id(2), args( Integer(0), Pointer, ) !dbg package_id=2 span=[105-109]
                Return !dbg package_id=2 span=[105-109]"#]],
    );
}

#[test]
fn unitary_call_within_a_repeat_until_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable idx = 0;
                repeat {
                    op(q);
                    set idx += 1;
                } until idx >= 3;
            }
        }
        "#,
    });

    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: op
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[162-165] scope=0
                Variable(1, Boolean) = Store Bool(true) !dbg package_id=2 span=[249-257] scope=1
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=1
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[223-226] scope=2 discriminator=1
                Variable(1, Boolean) = Store Bool(true) !dbg package_id=2 span=[249-257] scope=2 discriminator=1
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=2
                Variable(0, Integer) = Store Integer(2) !dbg package_id=2 span=[223-226] scope=2 discriminator=2
                Variable(1, Boolean) = Store Bool(true) !dbg package_id=2 span=[249-257] scope=2 discriminator=2
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=3
                Variable(0, Integer) = Store Integer(3) !dbg package_id=2 span=[223-226] scope=2 discriminator=3
                Variable(1, Boolean) = Store Bool(false) !dbg package_id=2 span=[249-257] scope=2 discriminator=3
                Call id(2), args( Integer(0), Pointer, ) !dbg package_id=2 span=[105-109]
                Return !dbg package_id=2 span=[105-109]"#]],
    );
}

#[test]
fn rotation_call_within_a_for_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation rotation(theta : Double, q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                for theta in [0.0, 1.0, 2.0] {
                    rotation(theta, q);
                }
            }
        }
        "#,
    });

    let rotation_callable_id = CallableId(1);
    assert_callable(
        &program,
        rotation_callable_id,
        &expect![[r#"
        Callable:
            name: rotation
            call_type: Regular
            input_type:
                [0]: Double
                [1]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[189-204] scope=1
                Call id(1), args( Double(0), Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=1
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[189-204] scope=2 discriminator=1
                Call id(1), args( Double(1), Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=2
                Variable(0, Integer) = Store Integer(2) !dbg package_id=2 span=[189-204] scope=2 discriminator=2
                Call id(1), args( Double(2), Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=3
                Variable(0, Integer) = Store Integer(3) !dbg package_id=2 span=[189-204] scope=2 discriminator=3
                Call id(2), args( Integer(0), Pointer, ) !dbg package_id=2 span=[127-131]
                Return !dbg package_id=2 span=[127-131]"#]],
    );
}

#[test]
fn rotation_call_within_a_while_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation rotation(theta : Double, q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let angles = [0.0, 1.0, 2.0];
                mutable idx = 0;
                while idx < 3 {
                    rotation(angles[idx], q);
                    set idx += 1;
                }
            }
        }
        "#,
    });

    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: rotation
            call_type: Regular
            input_type:
                [0]: Double
                [1]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[222-225] scope=0
                Call id(1), args( Double(0), Qubit(0), ) !dbg package_id=2 span=[0-0] scope=1 discriminator=1
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[309-312] scope=1 discriminator=1
                Call id(1), args( Double(1), Qubit(0), ) !dbg package_id=2 span=[0-0] scope=1 discriminator=2
                Variable(0, Integer) = Store Integer(2) !dbg package_id=2 span=[309-312] scope=1 discriminator=2
                Call id(1), args( Double(2), Qubit(0), ) !dbg package_id=2 span=[0-0] scope=1 discriminator=3
                Variable(0, Integer) = Store Integer(3) !dbg package_id=2 span=[309-312] scope=1 discriminator=3
                Call id(2), args( Integer(0), Pointer, ) !dbg package_id=2 span=[127-131]
                Return !dbg package_id=2 span=[127-131]"#]],
    );
}

#[test]
fn rotation_call_within_a_repeat_until_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation rotation(theta : Double, q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let angles = [0.0, 1.0, 2.0];
                mutable idx = 0;
                repeat {
                    rotation(angles[idx], q);
                    set idx += 1;
                } until idx >= 3;
            }
        }
        "#,
    });

    let rotation_callable_id = CallableId(1);
    assert_callable(
        &program,
        rotation_callable_id,
        &expect![[r#"
        Callable:
            name: rotation
            call_type: Regular
            input_type:
                [0]: Double
                [1]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[222-225] scope=0
                Variable(1, Boolean) = Store Bool(true) !dbg package_id=2 span=[328-336] scope=1
                Call id(1), args( Double(0), Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=1
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[302-305] scope=2 discriminator=1
                Variable(1, Boolean) = Store Bool(true) !dbg package_id=2 span=[328-336] scope=2 discriminator=1
                Call id(1), args( Double(1), Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=2
                Variable(0, Integer) = Store Integer(2) !dbg package_id=2 span=[302-305] scope=2 discriminator=2
                Variable(1, Boolean) = Store Bool(true) !dbg package_id=2 span=[328-336] scope=2 discriminator=2
                Call id(1), args( Double(2), Qubit(0), ) !dbg package_id=2 span=[0-0] scope=2 discriminator=3
                Variable(0, Integer) = Store Integer(3) !dbg package_id=2 span=[302-305] scope=2 discriminator=3
                Variable(1, Boolean) = Store Bool(false) !dbg package_id=2 span=[328-336] scope=2 discriminator=3
                Call id(2), args( Integer(0), Pointer, ) !dbg package_id=2 span=[127-131]
                Return !dbg package_id=2 span=[127-131]"#]],
    );
}

#[test]
fn mutable_bool_updated_in_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable flag = false;
                for _ in 1..3 {
                    if not flag {
                        set flag = MResetZ(q) == One;
                    }
                }
            }
        }
        "#,
    });

    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Variable(0, Boolean) = Store Bool(false) !dbg package_id=2 span=[107-111] scope=0
                Variable(1, Integer) = Store Integer(1) !dbg package_id=2 span=[138-142] scope=1
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[198-208] scope=1147 discriminator=1
                Variable(2, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[198-215] scope=3 discriminator=1
                Variable(3, Boolean) = Store Variable(2, Boolean) !dbg package_id=2 span=[198-215] scope=3 discriminator=1
                Variable(0, Boolean) = Store Variable(3, Boolean) !dbg package_id=2 span=[191-195] scope=3 discriminator=1
                Variable(1, Integer) = Store Integer(2) !dbg package_id=2 span=[138-142] scope=2 discriminator=1
                Variable(4, Boolean) = LogicalNot Variable(0, Boolean) !dbg package_id=2 span=[160-168] scope=2 discriminator=2
                Branch Variable(4, Boolean), 2, 1 !dbg package_id=2 span=[160-168] scope=2 discriminator=2"#]],
    );
}

#[test]
fn mutable_int_updated_in_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable count = 1;
                for _ in 1..3 {
                    if count > 0 and MResetZ(q) == One {
                        set count = -count;
                    }
                }
            }
        }
        "#,
    });

    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[107-112] scope=0
                Variable(1, Integer) = Store Integer(1) !dbg package_id=2 span=[135-139] scope=1
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[171-181] scope=1147 discriminator=1
                Variable(2, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[171-188] scope=2 discriminator=1
                Variable(3, Boolean) = Store Variable(2, Boolean) !dbg package_id=2 span=[171-188] scope=2 discriminator=1
                Branch Variable(3, Boolean), 2, 1 !dbg package_id=2 span=[157-188] scope=2 discriminator=1"#]],
    );
}

#[test]
fn mutable_double_updated_in_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable count = 1.1;
                for _ in 1..3 {
                    if count > 0.1 and MResetZ(q) == One {
                        set count = -count;
                    }
                }
            }
        }
        "#,
    });

    assert_blocks(
        &program,
        //BlockId(0),
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Variable(0, Double) = Store Double(1.1) !dbg package_id=2 span=[107-112] scope=0
                Variable(1, Integer) = Store Integer(1) !dbg package_id=2 span=[137-141] scope=1
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[175-185] scope=1147 discriminator=1
                Variable(2, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[175-192] scope=2 discriminator=1
                Variable(3, Boolean) = Store Variable(2, Boolean) !dbg package_id=2 span=[175-192] scope=2 discriminator=1
                Branch Variable(3, Boolean), 2, 1 !dbg package_id=2 span=[159-192] scope=2 discriminator=1
            Block 1:Block:
                Variable(1, Integer) = Store Integer(2) !dbg package_id=2 span=[137-141] scope=2 discriminator=1
                Variable(4, Boolean) = Fcmp Ogt, Variable(0, Double), Double(0.1) !dbg package_id=2 span=[159-170] scope=2 discriminator=2
                Variable(5, Boolean) = Store Bool(false) !dbg package_id=2 span=[175-192] scope=2 discriminator=2
                Branch Variable(4, Boolean), 4, 3 !dbg package_id=2 span=[175-192] scope=2 discriminator=2
            Block 2:Block:
                Variable(0, Double) = Store Double(-1.1) !dbg package_id=2 span=[215-220] scope=3 discriminator=1
                Jump(1) !dbg package_id=2 span=[193-244] scope=2 discriminator=1
            Block 3:Block:
                Branch Variable(5, Boolean), 6, 5 !dbg package_id=2 span=[159-192] scope=2 discriminator=2
            Block 4:Block:
                Call id(1), args( Qubit(0), Result(1), ) !dbg package_id=2 span=[175-185] scope=1147 discriminator=2
                Variable(6, Boolean) = Call id(2), args( Result(1), ) !dbg package_id=2 span=[175-192] scope=2 discriminator=2
                Variable(7, Boolean) = Store Variable(6, Boolean) !dbg package_id=2 span=[175-192] scope=2 discriminator=2
                Variable(5, Boolean) = Store Variable(7, Boolean) !dbg package_id=2 span=[175-192] scope=2 discriminator=2
                Jump(3) !dbg package_id=2 span=[175-192] scope=2 discriminator=2
            Block 5:Block:
                Variable(1, Integer) = Store Integer(3) !dbg package_id=2 span=[137-141] scope=2 discriminator=2
                Variable(9, Boolean) = Fcmp Ogt, Variable(0, Double), Double(0.1) !dbg package_id=2 span=[159-170] scope=2 discriminator=3
                Variable(10, Boolean) = Store Bool(false) !dbg package_id=2 span=[175-192] scope=2 discriminator=3
                Branch Variable(9, Boolean), 8, 7 !dbg package_id=2 span=[175-192] scope=2 discriminator=3
            Block 6:Block:
                Variable(8, Double) = Fmul Double(-1), Variable(0, Double) !dbg package_id=2 span=[223-229] scope=3 discriminator=2
                Variable(0, Double) = Store Variable(8, Double) !dbg package_id=2 span=[215-220] scope=3 discriminator=2
                Jump(5) !dbg package_id=2 span=[193-244] scope=2 discriminator=2
            Block 7:Block:
                Branch Variable(10, Boolean), 10, 9 !dbg package_id=2 span=[159-192] scope=2 discriminator=3
            Block 8:Block:
                Call id(1), args( Qubit(0), Result(2), ) !dbg package_id=2 span=[175-185] scope=1147 discriminator=3
                Variable(11, Boolean) = Call id(2), args( Result(2), ) !dbg package_id=2 span=[175-192] scope=2 discriminator=3
                Variable(12, Boolean) = Store Variable(11, Boolean) !dbg package_id=2 span=[175-192] scope=2 discriminator=3
                Variable(10, Boolean) = Store Variable(12, Boolean) !dbg package_id=2 span=[175-192] scope=2 discriminator=3
                Jump(7) !dbg package_id=2 span=[175-192] scope=2 discriminator=3
            Block 9:Block:
                Variable(1, Integer) = Store Integer(4) !dbg package_id=2 span=[137-141] scope=2 discriminator=3
                Call id(3), args( Integer(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 10:Block:
                Variable(13, Double) = Fmul Double(-1), Variable(0, Double) !dbg package_id=2 span=[223-229] scope=3 discriminator=3
                Variable(0, Double) = Store Variable(13, Double) !dbg package_id=2 span=[215-220] scope=3 discriminator=3
                Jump(9) !dbg package_id=2 span=[193-244] scope=2 discriminator=3"#]],
    );
}

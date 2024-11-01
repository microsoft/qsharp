// Licensed under the MIT License.
use expect_test::expect;

use super::check;

#[test]
fn bounded_polymorphism_eq() {
    check(
        r#"
        namespace A {
            function Foo<'T: Eq>(a: 'T, b: 'T) : Bool {
                a == b
            }
        }
        "#,
        "",
        &expect![[r##"
            #8 55-69 "(a: 'T, b: 'T)" : (Param<"'T": 0>, Param<"'T": 0>)
            #9 56-61 "a: 'T" : Param<"'T": 0>
            #13 63-68 "b: 'T" : Param<"'T": 0>
            #20 77-115 "{\n                a == b\n            }" : Bool
            #22 95-101 "a == b" : Bool
            #23 95-96 "a" : Param<"'T": 0>
            #26 100-101 "b" : Param<"'T": 0>
        "##]],
    );
}

// TODO(sezna) figure out why the error message is duplicated here
#[test]
fn bounded_polymorphism_error_on_infinite_recursive_generic() {
    check(
        r#"
        namespace A {
            function Foo<'T: Exp['T, Int]>(a: 'T, b: Int) : 'T {
                a ^ b
            }
        }
        "#,
        "",
        &expect![[r##"
            #13 65-80 "(a: 'T, b: Int)" : (Param<"'T": 0>, Int)
            #14 66-71 "a: 'T" : Param<"'T": 0>
            #18 73-79 "b: Int" : Int
            #25 86-123 "{\n                a ^ b\n            }" : Param<"'T": 0>
            #27 104-109 "a ^ b" : Param<"'T": 0>
            #28 104-105 "a" : Param<"'T": 0>
            #31 108-109 "b" : Int
            Error(Type(Error(TyConversionError(RecursiveClassConstraint(RecursiveClassConstraintError { span: Span { lo: 52, hi: 55 }, name: "Exp" })))))
            Error(Type(Error(TyConversionError(RecursiveClassConstraint(RecursiveClassConstraintError { span: Span { lo: 52, hi: 55 }, name: "Exp" })))))
            Error(Type(Error(TyConversionError(RecursiveClassConstraint(RecursiveClassConstraintError { span: Span { lo: 52, hi: 55 }, name: "Exp" })))))
            Error(Type(Error(TyConversionError(RecursiveClassConstraint(RecursiveClassConstraintError { span: Span { lo: 52, hi: 55 }, name: "Exp" })))))
            Error(Type(Error(MissingClassExp("'T", Span { lo: 104, hi: 109 }))))
            Error(Type(Error(MissingClassExp("'T", Span { lo: 104, hi: 109 }))))
        "##]],
    );
}

#[test]
fn bounded_polymorphism_exp() {
    check(
        r#"
        namespace A {
            function Foo<'E, 'T: Exp['E, Int]>(a: 'E, b: Int) : 'T {
                a ^ b
            }
        }
        "#,
        "",
        &expect![[r##"
            #14 69-84 "(a: 'E, b: Int)" : (Param<"'E": 0>, Int)
            #15 70-75 "a: 'E" : Param<"'E": 0>
            #19 77-83 "b: Int" : Int
            #26 90-127 "{\n                a ^ b\n            }" : Param<"'E": 0>
            #28 108-113 "a ^ b" : Param<"'E": 0>
            #29 108-109 "a" : Param<"'E": 0>
            #32 112-113 "b" : Int
            Error(Type(Error(MissingClassExp("'E", Span { lo: 108, hi: 113 }))))
            Error(Type(Error(TyMismatch("'T", "'E", Span { lo: 108, hi: 113 }))))
        "##]],
    );
}

#[test]
fn bounded_polymorphism_example_should_fail() {
    check(
        r#"
        namespace A {
            function Foo<'T: Eq, 'O: Eq>(a: 'T, b: 'O) : Bool {
            // should fail because we can't compare two different types
                a == b
            }
        }
        "#,
        "",
        &expect![[r##"
            #10 63-77 "(a: 'T, b: 'O)" : (Param<"'T": 0>, Param<"'O": 1>)
            #11 64-69 "a: 'T" : Param<"'T": 0>
            #15 71-76 "b: 'O" : Param<"'O": 1>
            #22 85-195 "{\n            // should fail because we can't compare two different types\n                a == b\n            }" : Bool
            #24 175-181 "a == b" : Bool
            #25 175-176 "a" : Param<"'T": 0>
            #28 180-181 "b" : Param<"'O": 1>
            Error(Type(Error(TyMismatch("'T", "'O", Span { lo: 180, hi: 181 }))))
        "##]],
    );
}

// This test ensures that we show a pretty error for polymorphism bounds that are not supported
// yet.
#[test]
fn bounded_polymorphism_iter() {
    check(
        r#"
        namespace A {
            function Foo<'T: Iterable[Bool]>(a: 'T) : Bool {
                for item in a {
                    return item;
                }
            }

            function Main() : Unit {
                let x = Foo([true]);
            }
        }
        "#,
        "",
        &expect![[r##"
            #11 67-74 "(a: 'T)" : Param<"'T": 0>
            #12 68-73 "a: 'T" : Param<"'T": 0>
            #19 82-180 "{\n                for item in a {\n                    return item;\n                }\n            }" : Bool
            #21 100-166 "for item in a {\n                    return item;\n                }" : Bool
            #22 104-108 "item" : Bool
            #24 112-113 "a" : Param<"'T": 0>
            #27 114-166 "{\n                    return item;\n                }" : Unit
            #29 136-147 "return item" : Unit
            #30 143-147 "item" : Bool
            #36 207-209 "()" : Unit
            #40 217-269 "{\n                let x = Foo([true]);\n            }" : Unit
            #42 239-240 "x" : Bool
            #44 243-254 "Foo([true])" : Bool
            #45 243-246 "Foo" : (Bool[] -> Bool)
            #48 246-254 "([true])" : Bool[]
            #49 247-253 "[true]" : Bool[]
            #50 248-252 "true" : Bool
            Error(Type(Error(UnsupportedParametricClassBound(Span { lo: 112, hi: 113 }))))
            "##]],
    );
}

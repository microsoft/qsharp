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

#[test]
fn bounded_polymorphism_num() {
    check(
        r#"
        namespace A {
            function Foo<'T: Num>(a: 'T) : 'T {
                -a
            }

            function Main() : Unit {
                let x: Int = Foo(1);
                let y: Double = Foo(1.0);
                let z: BigInt = Foo(10L);
            }
        }
        "#,
        "",
        &expect![[r##"
            #8 56-63 "(a: 'T)" : Param<"'T": 0>
            #9 57-62 "a: 'T" : Param<"'T": 0>
            #15 69-103 "{\n                -a\n            }" : Param<"'T": 0>
            #17 87-89 "-a" : Param<"'T": 0>
            #18 88-89 "a" : Param<"'T": 0>
            #24 130-132 "()" : Unit
            #28 140-276 "{\n                let x: Int = Foo(1);\n                let y: Double = Foo(1.0);\n                let z: BigInt = Foo(10L);\n            }" : Unit
            #30 162-168 "x: Int" : Int
            #35 171-177 "Foo(1)" : Int
            #36 171-174 "Foo" : (Int -> Int)
            #39 174-177 "(1)" : Int
            #40 175-176 "1" : Int
            #42 199-208 "y: Double" : Double
            #47 211-219 "Foo(1.0)" : Double
            #48 211-214 "Foo" : (Double -> Double)
            #51 214-219 "(1.0)" : Double
            #52 215-218 "1.0" : Double
            #54 241-250 "z: BigInt" : BigInt
            #59 253-261 "Foo(10L)" : BigInt
            #60 253-256 "Foo" : (BigInt -> BigInt)
            #63 256-261 "(10L)" : BigInt
            #64 257-260 "10L" : BigInt
        "##]],
    );
}

#[test]
fn bounded_polymorphism_num_fail() {
    check(
        r#"
        namespace A {
            function Foo<'T: Eq>(a: 'T) : 'T {
                -a
            }

            function Main() : Unit {
                let x: Int = Foo(1);
                let y: Double = Foo(1.0);
                let z: BigInt = Foo(10L);
            }
        }
        "#,
        "",
        &expect![[r##"
            #8 55-62 "(a: 'T)" : Param<"'T": 0>
            #9 56-61 "a: 'T" : Param<"'T": 0>
            #15 68-102 "{\n                -a\n            }" : Param<"'T": 0>
            #17 86-88 "-a" : Param<"'T": 0>
            #18 87-88 "a" : Param<"'T": 0>
            #24 129-131 "()" : Unit
            #28 139-275 "{\n                let x: Int = Foo(1);\n                let y: Double = Foo(1.0);\n                let z: BigInt = Foo(10L);\n            }" : Unit
            #30 161-167 "x: Int" : Int
            #35 170-176 "Foo(1)" : Int
            #36 170-173 "Foo" : (Int -> Int)
            #39 173-176 "(1)" : Int
            #40 174-175 "1" : Int
            #42 198-207 "y: Double" : Double
            #47 210-218 "Foo(1.0)" : Double
            #48 210-213 "Foo" : (Double -> Double)
            #51 213-218 "(1.0)" : Double
            #52 214-217 "1.0" : Double
            #54 240-249 "z: BigInt" : BigInt
            #59 252-260 "Foo(10L)" : BigInt
            #60 252-255 "Foo" : (BigInt -> BigInt)
            #63 255-260 "(10L)" : BigInt
            #64 256-259 "10L" : BigInt
            Error(Type(Error(MissingClassNum("'T", Span { lo: 87, hi: 88 }))))
        "##]],
    );
}

#[test]
fn transitive_class_check() {
    check(
        r#"
        namespace A {
            function Foo<'T: Num>(a: 'T) : 'T {
                -a
            }

            function Bar<'F: Num>(a: 'F) : 'F {
                Foo(a)
            }

            function Main() : Unit {
                let x: Int = Bar(1);
                let y: Double = Bar(1.0);
                let z: BigInt = Bar(10L);
            }
        }
        "#,
        "",
        &expect![[r##"
            #8 56-63 "(a: 'T)" : Param<"'T": 0>
            #9 57-62 "a: 'T" : Param<"'T": 0>
            #15 69-103 "{\n                -a\n            }" : Param<"'T": 0>
            #17 87-89 "-a" : Param<"'T": 0>
            #18 88-89 "a" : Param<"'T": 0>
            #26 138-145 "(a: 'F)" : Param<"'F": 0>
            #27 139-144 "a: 'F" : Param<"'F": 0>
            #33 151-189 "{\n                Foo(a)\n            }" : Param<"'F": 0>
            #35 169-175 "Foo(a)" : Param<"'F": 0>
            #36 169-172 "Foo" : (Param<"'F": 0> -> Param<"'F": 0>)
            #39 172-175 "(a)" : Param<"'F": 0>
            #40 173-174 "a" : Param<"'F": 0>
            #46 216-218 "()" : Unit
            #50 226-362 "{\n                let x: Int = Bar(1);\n                let y: Double = Bar(1.0);\n                let z: BigInt = Bar(10L);\n            }" : Unit
            #52 248-254 "x: Int" : Int
            #57 257-263 "Bar(1)" : Int
            #58 257-260 "Bar" : (Int -> Int)
            #61 260-263 "(1)" : Int
            #62 261-262 "1" : Int
            #64 285-294 "y: Double" : Double
            #69 297-305 "Bar(1.0)" : Double
            #70 297-300 "Bar" : (Double -> Double)
            #73 300-305 "(1.0)" : Double
            #74 301-304 "1.0" : Double
            #76 327-336 "z: BigInt" : BigInt
            #81 339-347 "Bar(10L)" : BigInt
            #82 339-342 "Bar" : (BigInt -> BigInt)
            #85 342-347 "(10L)" : BigInt
            #86 343-346 "10L" : BigInt
        "##]],
    );
}

#[test]
fn transitive_class_check_fail() {
    check(
        r#"
        namespace A {
            function Foo<'T: Integral>(a: 'T) : 'T {
                a
            }

            function Bar<'F>(a: 'F) : 'F {
                // below should be an error as 'F has no
                // Integral bound
                Foo(a)
            }

            function Main() : Unit {
                let x: Int = Foo(1);
                // below should be an error as it is a double and not an integral type
                let y: Double = Foo(1.0);
                let z: BigInt = Foo(10L);
            }
        }
        "#,
        "",
        &expect![[r##"
            #8 61-68 "(a: 'T)" : Param<"'T": 0>
            #9 62-67 "a: 'T" : Param<"'T": 0>
            #15 74-107 "{\n                a\n            }" : Param<"'T": 0>
            #17 92-93 "a" : Param<"'T": 0>
            #24 137-144 "(a: 'F)" : Param<"'F": 0>
            #25 138-143 "a: 'F" : Param<"'F": 0>
            #31 150-279 "{\n                // below should be an error as 'F has no\n                // Integral bound\n                Foo(a)\n            }" : Param<"'F": 0>
            #33 259-265 "Foo(a)" : Param<"'F": 0>
            #34 259-262 "Foo" : (Param<"'F": 0> -> Param<"'F": 0>)
            #37 262-265 "(a)" : Param<"'F": 0>
            #38 263-264 "a" : Param<"'F": 0>
            #44 306-308 "()" : Unit
            #48 316-539 "{\n                let x: Int = Foo(1);\n                // below should be an error as it is a double and not an integral type\n                let y: Double = Foo(1.0);\n                let z: BigInt = Foo(10L);\n            }" : Unit
            #50 338-344 "x: Int" : Int
            #55 347-353 "Foo(1)" : Int
            #56 347-350 "Foo" : (Int -> Int)
            #59 350-353 "(1)" : Int
            #60 351-352 "1" : Int
            #62 462-471 "y: Double" : Double
            #67 474-482 "Foo(1.0)" : Double
            #68 474-477 "Foo" : (Double -> Double)
            #71 477-482 "(1.0)" : Double
            #72 478-481 "1.0" : Double
            #74 504-513 "z: BigInt" : BigInt
            #79 516-524 "Foo(10L)" : BigInt
            #80 516-519 "Foo" : (BigInt -> BigInt)
            #83 519-524 "(10L)" : BigInt
            #84 520-523 "10L" : BigInt
            Error(Type(Error(MissingClassInteger("'F", Span { lo: 259, hi: 265 }))))
            Error(Type(Error(MissingClassInteger("Double", Span { lo: 474, hi: 482 }))))
        "##]],
    );
}

#[test]
fn transitive_class_check_superset() {
    check(
        r#"
        namespace A {
            function Foo<'T: Num>(a: 'T) : 'T {
                -a
            }

            function Bar<'F: Num + Eq>(a: 'F) : 'F {
                Foo(a)
            }

            function Main() : Unit {
                let x: Int = Bar(1);
                let y: Double = Bar(1.0);
                let z: BigInt = Bar(10L);
            }
        }
        "#,
        "",
        &expect![[r##"
            #8 56-63 "(a: 'T)" : Param<"'T": 0>
            #9 57-62 "a: 'T" : Param<"'T": 0>
            #15 69-103 "{\n                -a\n            }" : Param<"'T": 0>
            #17 87-89 "-a" : Param<"'T": 0>
            #18 88-89 "a" : Param<"'T": 0>
            #27 143-150 "(a: 'F)" : Param<"'F": 0>
            #28 144-149 "a: 'F" : Param<"'F": 0>
            #34 156-194 "{\n                Foo(a)\n            }" : Param<"'F": 0>
            #36 174-180 "Foo(a)" : Param<"'F": 0>
            #37 174-177 "Foo" : (Param<"'F": 0> -> Param<"'F": 0>)
            #40 177-180 "(a)" : Param<"'F": 0>
            #41 178-179 "a" : Param<"'F": 0>
            #47 221-223 "()" : Unit
            #51 231-367 "{\n                let x: Int = Bar(1);\n                let y: Double = Bar(1.0);\n                let z: BigInt = Bar(10L);\n            }" : Unit
            #53 253-259 "x: Int" : Int
            #58 262-268 "Bar(1)" : Int
            #59 262-265 "Bar" : (Int -> Int)
            #62 265-268 "(1)" : Int
            #63 266-267 "1" : Int
            #65 290-299 "y: Double" : Double
            #70 302-310 "Bar(1.0)" : Double
            #71 302-305 "Bar" : (Double -> Double)
            #74 305-310 "(1.0)" : Double
            #75 306-309 "1.0" : Double
            #77 332-341 "z: BigInt" : BigInt
            #82 344-352 "Bar(10L)" : BigInt
            #83 344-347 "Bar" : (BigInt -> BigInt)
            #86 347-352 "(10L)" : BigInt
            #87 348-351 "10L" : BigInt
        "##]],
    );
}
#[test]
fn bounded_polymorphism_show() {
    check(
        r#"
        namespace A {
            function Foo<'T: Show>(a: 'T) : String {
                let x = $"Value: {a}";
                x
            }

            function Main() : Unit {
                let x: String = Foo(1);
                let y: String = Foo(1.0);
                let z: String = Foo(true);
            }
        }
        "#,
        "",
        &expect![[r##"
            #8 57-64 "(a: 'T)" : Param<"'T": 0>
            #9 58-63 "a: 'T" : Param<"'T": 0>
            #16 74-146 "{\n                let x = $\"Value: {a}\";\n                x\n            }" : String
            #18 96-97 "x" : String
            #20 100-113 "$\"Value: {a}\"" : String
            #21 110-111 "a" : Param<"'T": 0>
            #25 131-132 "x" : String
            #31 173-175 "()" : Unit
            #35 183-323 "{\n                let x: String = Foo(1);\n                let y: String = Foo(1.0);\n                let z: String = Foo(true);\n            }" : Unit
            #37 205-214 "x: String" : String
            #42 217-223 "Foo(1)" : String
            #43 217-220 "Foo" : (Int -> String)
            #46 220-223 "(1)" : Int
            #47 221-222 "1" : Int
            #49 245-254 "y: String" : String
            #54 257-265 "Foo(1.0)" : String
            #55 257-260 "Foo" : (Double -> String)
            #58 260-265 "(1.0)" : Double
            #59 261-264 "1.0" : Double
            #61 287-296 "z: String" : String
            #66 299-308 "Foo(true)" : String
            #67 299-302 "Foo" : (Bool -> String)
            #70 302-308 "(true)" : Bool
            #71 303-307 "true" : Bool
        "##]],
    );
}

#[test]
fn bounded_polymorphism_show_fail() {
    check(
        r#"
        namespace A {
            function Foo<'T>(a: 'T) : String {
                Message($"Value: {a}")
            }

            function Main() : Unit {
                let x = Foo(1);
                let y = Foo(1.0);
                let z = Foo(true);
            }
        }
        "#,
        "",
        &expect![[r##"
            #7 51-58 "(a: 'T)" : Param<"'T": 0>
            #8 52-57 "a: 'T" : Param<"'T": 0>
            #15 68-122 "{\n                Message($\"Value: {a}\")\n            }" : String
            #17 86-108 "Message($\"Value: {a}\")" : String
            #18 86-93 "Message" : ?
            #21 93-108 "($\"Value: {a}\")" : String
            #22 94-107 "$\"Value: {a}\"" : String
            #23 104-105 "a" : Param<"'T": 0>
            #29 149-151 "()" : Unit
            #33 159-275 "{\n                let x = Foo(1);\n                let y = Foo(1.0);\n                let z = Foo(true);\n            }" : Unit
            #35 181-182 "x" : String
            #37 185-191 "Foo(1)" : String
            #38 185-188 "Foo" : (Int -> String)
            #41 188-191 "(1)" : Int
            #42 189-190 "1" : Int
            #44 213-214 "y" : String
            #46 217-225 "Foo(1.0)" : String
            #47 217-220 "Foo" : (Double -> String)
            #50 220-225 "(1.0)" : Double
            #51 221-224 "1.0" : Double
            #53 247-248 "z" : String
            #55 251-260 "Foo(true)" : String
            #56 251-254 "Foo" : (Bool -> String)
            #59 254-260 "(true)" : Bool
            #60 255-259 "true" : Bool
            Error(Resolve(NotFound("Message", Span { lo: 86, hi: 93 })))
            Error(Type(Error(MissingClassShow("'T", Span { lo: 104, hi: 105 }))))
        "##]],
    );
}

#[test]
fn bounded_polymorphism_integral() {
    check(
        r#"
        namespace A {
            function Foo<'T: Integral>(a: 'T) : 'T {
                a ^^^ a
            }

            function Main() : Unit {
                let x: Int = Foo(1);
                let y: BigInt = Foo(10L);
            }
        }
        "#,
        "",
        &expect![[r##""##]],
    );
}
#[test]
fn bounded_polymorphism_integral_fail() {
    check(
        r#"
        namespace A {
            function Foo<'T: Integral>(a: 'T) : 'T {
                a ^^^ a
            }

            function Main() : Unit {
                let x = Foo(1.0);
                let y = Foo(true);
            }
        }
        "#,
        "",
        &expect![[r##""##]],
    );
}

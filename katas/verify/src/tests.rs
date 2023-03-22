// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//use std::fs;
use indoc::indoc;

use std::env::current_dir;
use std::path::Path;
use std::path::PathBuf;
//use relative_path::RelativePath;

use crate::{verify_kata};

fn katas_source_dir() -> PathBuf {
    let current_dir = current_dir().unwrap();
    let katas_qsharp_source_dir = current_dir.parent().unwrap().join("qs");
    katas_qsharp_source_dir.to_path_buf()
}

fn verify_exercise() {
    //let root = current_dir();
    let path = Path::new("../../qs/single_qubit_gates/task_01/reference.qs");
    //let full_path = relative_path.to_path(&root);
    //println!("{}: {}", path.canonicalize().expect("Something").display(), path.exists());
    let katas_source = katas_source_dir();
    println!("{:?}", katas_source);
    println!("{:?}", path.canonicalize());
    println!("{:?}", std::env::current_dir().expect("cesarzc: no-current-dir").canonicalize());
    //let data = fs::read_to_string("../../qs/single_qubit_gates/task_01/reference.qs").expect("Unable to read file");
    //println!("{}", data);
}

#[test]
fn verify_single_qubit_gates_kata() {
    verify_exercise();
    verify_kata(
        indoc! {"
        namespace Quantum.Kata.SingleQubitGates {
            open Microsoft.Quantum.Diagnostics;
            open Microsoft.Quantum.Intrinsic;

            operation ApplyYReference(q : Qubit) : Unit is Adj + Ctl {
                body ... {
                    Y(q);
                }
                adjoint self;
            }

            operation Verify() : Bool {
                let task = ApplyY;
                let taskRef = ApplyYReference;
            
                use (aux, target) = (Qubit(), Qubit());
                H(aux);
                CNOT(aux, target);
            
                task(target);
                Adjoint taskRef(target);
            
                CNOT(aux, target);
                H(aux);
            
                if CheckZero(target) {
                    if CheckZero(aux) {
                        task(target);
                        DumpMachine();
                        return true;
                    }
                }

                //Reset(aux);
                //Reset(target);

                // Use DumpMachine to display actual vs desired state.
                task(target);
                DumpMachine();
                //Reset(target);
                taskRef(target);
                DumpMachine();

                return false;
            }
        }"},
        indoc! {"
        namespace Quantum.Kata.SingleQubitGates {
            open Microsoft.Quantum.Intrinsic;
            operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
                // Apply the Pauli Y operation.
                Y(q);
            }
        }"});
}
use crate::vm::VM;
use std::fs;

mod chunks;
mod compiler;
mod memory;
mod object;
mod scanner;
mod value;
mod vm;

fn main() -> std::io::Result<()> {
    let mut vm = VM::new();
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/2plus2.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/benchmark.lox",
    // )?; //NOT OK Operands must be numbers
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/block1.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/block2.lox",
    // )?; //OK
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/class_0.lox",
    // )?; // NOT OK Undefined property doStaff
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/class_1.lox",
    // )?; //NOT OK Expected 0 arguments, got 1.
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/class_2.lox",
    // )?; //NOT OK Undefined property doStaff
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/class_3.lox",
    // )?; //NOT OK Expected 0 arguments, got 1.
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/class_4.lox",
    // )?; //NOT OK
    let contents = fs::read_to_string(
        "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/class_5.lox",
    )?; //NOT OK Expected class
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/closure.lox",
    // )?; // NOT OK index out of bounds: the len is 2 but the index is 2
    //  let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/echo.lox",
    // )?; NOT OK
    //  let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/example1.lox",
    // )?; OK
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/example2.lox",
    // )?; OK
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/expresions.lox",
    // )?; OK
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/fib1.lox",
    // )?; OK
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/fib2.lox",
    // )?; OK
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/fib3.lox",
    // )?;  NOT OK
    // let contents = fs::read_to_string(
    //     "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/for.lox",
    // )?; OK
    vm.interpret(contents.to_owned());

    Ok(())
}
// TODO after all is OK, test all lox scripts again

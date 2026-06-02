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
    let scripts_location = "/mnt/d/Projects/crafting_interpreters/compiler_in_rust/lox_scripts/";
    let mut vm = VM::new();
    let contents = fs::read_to_string(scripts_location.to_owned() + "plus_equal.lox")?; //NOT  OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "2plus2.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "benchmark.lox",
    // )?; // NOT OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "block1.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "block2.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "class_0.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "class_1.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "class_2.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "class_3.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "class_4.lox",
    // )?; // NOT OK Class methods don't work
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "class_5.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "closure.lox",
    // )?; // OK
    //  let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "echo.lox",
    // )?; // OK
    //  let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "example1.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "example2.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "expresions.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "fib1.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "fib2.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "fib3.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "for.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "while.lox",
    // )?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "while_block.lox",
    // )?; // OK
    // let contents = fs::read_to_string(scripts_location.to_owned() + "recursion.lox")?; // OK
    // let contents = fs::read_to_string(
    //     scripts_location.to_owned() + "/test.lox",
    // )?; // OK
    vm.interpret(contents.to_owned());

    Ok(())
}

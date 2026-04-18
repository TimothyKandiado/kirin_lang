use compiler::{ir::lower_ast, lexer::parse_tokens, parser::parse_ast, program::build_program, type_check::TypeChecker};
use program::{Constant, FunctionKind, Program};
use runtime::{VM, native::get_native_functions};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        print_help();
        return;
    }

    let filename = &args[1];

    if filename.ends_with(".kin") {
        compile_and_run(filename);
    } else if filename.ends_with(".knb") {
        run(filename);
    } else {
        println!("{} is not valid file, expected '.kin' or '.knb' files", filename);
    }
}

fn compile_and_run(name: &str) {
    let source = std::fs::read_to_string(name).unwrap();

    let results = parse_tokens(&source);
    if let Err(errors) = results {
        for error in errors {
            println!("{}", error);
        }

        return;
    }

    let tokens = results.unwrap();

    let ast_result = parse_ast(tokens);

    if let Err(errors) = ast_result {
        for error in errors {
            println!(
                "[Parse Error][line: {}, column: {}] '{}'",
                error.line, error.column, error.context
            )
        }
        return;
    }

    let mut ast = ast_result.unwrap();

    let type_checker = TypeChecker::new();

    let errors = type_checker.check_module(&mut ast);

    if !errors.is_empty() {
        for error in errors {
            println!(
                "[Type Error][line: {}, column: {}] '{}'",
                error.line, error.column, error.context
            )
        }
    }

    let ir_module = lower_ast(&ast);

    let mut program = build_program(ir_module);

    let native_functions = get_native_functions();

    for function in program.functions.iter_mut() {
        if function.function_kind != FunctionKind::Native {
            continue;
        }

        let name_const = &program.constants[function.name_idx as usize];

        let Constant::String(name) = name_const else {
            panic!("name for native function does not exist")
        };

        let native_fn_index =
            native_functions
                .iter()
                .position(|f| f.name == name)
                .unwrap_or_else(|| panic!("native function with name {} does not exist",
                    name));

        function.code_offset = native_fn_index as u32
    }

    let mut vm = VM::new(&program, &native_functions);
    vm.run().unwrap()
}

fn run(name: &str) {
    let program_data = std::fs::read(name).unwrap();

    let mut program = Program::read_from_bytes(&program_data).unwrap();

    let native_functions = get_native_functions();

    for function in program.functions.iter_mut() {
        if function.function_kind != FunctionKind::Native {
            continue;
        }

        let name_const = &program.constants[function.name_idx as usize];

        let Constant::String(name) = name_const else {
            panic!("name for native function does not exist")
        };

        let native_fn_index =
            native_functions
                .iter()
                .position(|f| f.name == name)
                .unwrap_or_else(|| panic!("native function with name {} does not exist",
                    name));

        function.code_offset = native_fn_index as u32
    }

    let mut vm = VM::new(&program, &native_functions);
    vm.run().unwrap()
}

fn print_help() {
    let version_major: u8 = env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap();
    let version_minor: u8 = env!("CARGO_PKG_VERSION_MINOR").parse().unwrap();
    let version_patch: u8 = env!("CARGO_PKG_VERSION_PATCH").parse().unwrap();
    println!("Kirin language runtime v{}.{}.{}", version_major, version_minor, version_patch);
    println!("Usage: kirin [file]")
}

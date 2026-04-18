use program::{Constant, FunctionKind, Program};
use runtime::{VM, native::get_native_functions};

fn main() {
    let program_data = std::fs::read("samples/hello.knb").unwrap();

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

        let native_fn_index = native_functions
            .iter()
            .position(|f| f.name == name)
            .unwrap_or_else(|| panic!("native function with name {} does not exist", name));

        function.code_offset = native_fn_index as u32
    }

    // println!("successfully loaded program");

    //debug_program(&program);

    let mut vm = VM::new(&program, &native_functions);
    vm.run().unwrap()
}

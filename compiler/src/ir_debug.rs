use crate::{ir::{IrBlock, IrFunction, IrModule}, parser::format_type};

fn print_indent(size: usize) {
    for _ in 0..size {
        print!(" ")
    }
}

fn println(indent: usize, text: String) {
    print_indent(indent);
    println!("{}", text);
}

fn print_block(indent: usize, block: &IrBlock) {
    println(indent, format!("block {}:", block.label));

    for inst in &block.instructions {
        println(indent + 2, format!("{:?}", inst));
    }
}

pub fn debug_ir_module<'a>(ir_module: &'a IrModule<'a>) {
    println(0, "==== Ir Module ====".to_string());
    println(2, format!("package: {}", ir_module.package_name));
    println(2, format!("file: {}", ir_module.file_name));

    println(0, "==== Globals ====".to_string());
    for (name, value) in &ir_module.globals {
        println(2, format!("[{}] = {}", name, value));
    }

    println(0, "==== Functions ====".to_string());

    for function in &ir_module.functions {
        match function {
            IrFunction::Native {
                name,
                params,
                ret_type,
            } => {
                let params = params.iter().map(|param| {param.to_string()}).collect::<Vec<String>>().join(", ");
                println(
                2,
                format!("native fn {} ({}) : {}", name, params, ret_type.to_string()),
            );
            },
            IrFunction::Bytecode {
                name,
                params,
                ret_type,
                blocks,
                reg_count: _,
                reg_types,
            } => {
                let params = params.iter().map(|param| {param.to_string()}).collect::<Vec<String>>().join(", ");
                println(2, format!("fn {} ({}) : {}", name, params, ret_type.to_string()));

                println(4, "registers:".to_string());
                for (index, reg) in reg_types.iter().enumerate() {
                    println(6, format!("[{}] : {}", index, reg.to_string()))
                }

                for block in blocks {
                    print_block(4, block);
                }
            }
        }

        println!()
    }
}

use program::{Constant, TypeKind};

use crate::{Register, vm::VmContext, vm::VmError};

pub type NativeFunc =
    fn(ctx: &mut VmContext<'_>, args: &[Register], ret: &mut [Register]) -> Result<(), VmError>;

pub struct NativeFunctionWrapper {
    pub name: &'static str,
    pub function: NativeFunc,
}

pub fn get_native_functions() -> Vec<NativeFunctionWrapper> {
    let functions = vec![
        NativeFunctionWrapper {
            name: "print_i64",
            function: print_i64,
        },
        NativeFunctionWrapper {
            name: "print_str",
            function: print_str,
        },
        NativeFunctionWrapper {
            name: "print_any",
            function: print_any,
        },
    ];

    functions
}

fn print_i64(_: &mut VmContext<'_>, args: &[Register], _: &mut [Register]) -> Result<(), VmError> {
    if args.len() != 1 {
        return Err(VmError {
            message: format!("expected 1 argument but found {} instead", args.len()),
        });
    }

    println!("{}", args[0] as i64);

    Ok(())
}

fn print_str(
    ctx: &mut VmContext<'_>,
    args: &[Register],
    _: &mut [Register],
) -> Result<(), VmError> {
    if args.len() != 1 {
        return Err(VmError {
            message: format!("expected 1 argument but found {} instead", args.len()),
        });
    }

    let constant = &ctx.constants[args[0] as usize];

    if let Constant::String(str) = constant {
        println!("{}", str);
    } else {
        return Err(VmError {
            message: format!("expected string but found {:?}", constant),
        });
    }

    Ok(())
}

fn print_any(
    ctx: &mut VmContext<'_>,
    args: &[Register],
    _: &mut [Register],
) -> Result<(), VmError> {
    if args.len() != 2 {
        return Err(VmError {
            message: "expected 2 registers as arguments".to_string(),
        });
    }

    let type_info = ctx.types[args[0] as usize];

    match type_info.kind {
        TypeKind::I64 => {
            let value = args[1] as i64;

            println!("{}", value);
        }

        TypeKind::F64 => {
            let value = f64::from_bits(args[1]);

            println!("{}", value);
        }

        TypeKind::Bool => {
            let value = if args[1] == 0 { "true" } else { "false" };

            println!("{}", value)
        }

        TypeKind::String => {
            let const_index = args[1] as usize;

            if let Constant::String(string) = &ctx.constants[const_index] {
                println!("{}", string);
            } else {
                return Err(VmError {
                    message: format!(
                        "expected constant string but found {:?}",
                        &ctx.constants[const_index]
                    ),
                });
            }
        }
    }

    Ok(())
}

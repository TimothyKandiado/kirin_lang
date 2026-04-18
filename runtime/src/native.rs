use program::Constant;

use crate::{Register, VmContext, VmError};

pub type NativeFunc = fn(ctx: &mut VmContext<'_>, args: &[Register], ret: &mut [Register]) -> Result<(), VmError>;

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

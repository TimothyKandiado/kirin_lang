use compiler::{ir::lower_ast, ir_debug::debug_ir_module, lexer::parse_tokens, parser::parse_ast};

fn main() {
    let source = std::fs::read_to_string("../samples/hello.kin").unwrap();

    let results = parse_tokens(&source);

    if let Err(errors) = results {
        for error in errors {
            println!("{}", error);
        }

        return;
    }

    let tokens = results.unwrap();

    for token in &tokens {
        println!("{:?}", token)
    }

    let ast_result = parse_ast(tokens);

    if let Err(errors) = ast_result {
        for error in errors {
            println!(
                "[Error][line: {}, column: {}] '{}'",
                error.line, error.column, error.context
            )
        }
        return;
    }

    let ast = ast_result.unwrap();

    for stmt in &ast {
        println!("{:?}", stmt);
    }

    let ir_module = lower_ast(&ast);
    debug_ir_module(&ir_module)
}

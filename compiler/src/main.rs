use compiler::{lexer::parse_tokens, parser::parse_ast};

fn main() {
    let source = std::fs::read_to_string("../samples/hello.kin").unwrap();

    let results = parse_tokens(&source);

    if results.is_err() {
        let errors = results.unwrap_err();

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

    if ast_result.is_err() {
        let errors = ast_result.unwrap_err();

        for error in errors {
            println!(
                "[Error][line: {}, column: {}] '{}'",
                error.line, error.column, error.context
            )
        }
        return;
    }

    let ast = ast_result.unwrap();

    for stmt in ast {
        println!("{:?}", stmt);
    }
}

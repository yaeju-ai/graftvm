fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut debug = false;
    let mut dump_only = false;
    let mut rest: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--debug" => debug = true,
            "--dump" => dump_only = true,
            other => rest.push(other.to_string()),
        }
        i += 1;
    }

    if rest.is_empty() {
        repl(debug);
        return;
    }

    let source = if rest[0] == "-e" {
        rest[1..].join(" ")
    } else {
        match std::fs::read_to_string(&rest[0]) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: could not read '{}': {}", rest[0], e);
                std::process::exit(1);
            }
        }
    };

    let tokens = graftvm_language::lexer::lex(&source);
    let parsed = graftvm_language::parser::parse(&tokens).unwrap();
    let bytecode = graftvm_language::lower::lower(parsed).unwrap();

    if dump_only {
        println!(";; bytecode: {} instructions", bytecode.len());
        for (i, op) in bytecode.iter().enumerate() {
            println!("  [{:>3}] {:?}", i, op);
        }
        return;
    }

    match run(&source, debug) {
        Ok(bc) => {
            if debug {
                println!(";; bytecode: {} instructions", bc.len());
                for (i, op) in bc.iter().enumerate() {
                    println!("  [{:>3}] {:?}", i, op);
                }
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run(source: &str, debug: bool) -> Result<Vec<graftvm_bytecode::Opcode>, String> {
    let tokens = graftvm_language::lexer::lex(source);
    let parsed = graftvm_language::parser::parse(&tokens)?;
    let bytecode = graftvm_language::lower::lower(parsed)?;

    let mut vm = graftvm_engine::vm::VM::new(bytecode.clone());

    if debug {
        println!(";; ── execution trace ──");
        vm.run_debug()
    } else {
        vm.run()
    }
    .map_err(|e| format!("runtime: {}", e))?;

    vm.dump_state();

    Ok(bytecode)
}

fn repl(debug: bool) {
    println!("Parlance v0.1 — GraftVM frontend language");
    println!("Type an expression, or :quit to exit.");
    use std::io::Write;

    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() || line.trim() == ":quit" {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match run(line, debug) {
            Ok(_) => {}
            Err(e) => println!(";; error: {}", e),
        }
    }
}

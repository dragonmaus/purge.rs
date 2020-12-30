use getopt::Opt;

program::main!("purge");

fn usage_line(program_name: &str) -> String {
    format!("Usage: {} [-h] path [path ...]", program_name)
}

fn print_usage(program_name: &str) {
    println!("{}", usage_line(program_name));
    println!("  -h   display this help");
}

fn program(name: &str) -> program::Result {
    let mut args = program::args();
    let mut opts = getopt::Parser::new(&args, "h");

    loop {
        match opts.next().transpose()? {
            None => break,
            Some(opt) => match opt {
                Opt('h', None) => {
                    print_usage(name);
                    return Ok(0);
                }
                _ => unreachable!(),
            },
        }
    }

    let args = args.split_off(opts.index());
    if args.is_empty() {
        eprintln!("{}", usage_line(name));
        return Ok(1);
    }

    for arg in args {
        purge::purge(&arg)?;
    }

    Ok(0)
}

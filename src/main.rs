use getopt::Opt;

program::main!("purge");

fn usage_line(program_name: &str) -> String {
    format!("Usage: {} [-h] [-v[v]] path [path ...]", program_name)
}

fn print_usage(program_name: &str) {
    println!("{}", usage_line(program_name));
    println!("  -v   show files as they are removed");
    println!("       use twice to also show progress");
    println!("  -h   display this help");
}

fn program(name: &str) -> program::Result {
    let mut args = program::args();
    let mut opts = getopt::Parser::new(&args, "hv");
    let mut verbosity = 0;

    loop {
        match opts.next().transpose()? {
            None => break,
            Some(opt) => match opt {
                Opt('v', None) => {
                    if verbosity < 2 {
                        verbosity += 1;
                    }
                }
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
        purge::purge(&arg, verbosity)?;
    }

    Ok(0)
}

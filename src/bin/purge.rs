use getopt::Opt;

program::main!("purge");

fn usage_line() -> String {
    format!("Usage: {} [-h] path [path ...]", program::name("purge"))
}

fn print_usage() {
    println!("{}", usage_line());
    println!("  -h   display this help");
}

fn program() -> program::Result {
    let mut args = program::args();
    let mut opts = getopt::Parser::new(&args, "h");

    loop {
        match opts.next().transpose()? {
            None => break,
            Some(opt) => match opt {
                Opt('h', None) => {
                    print_usage();
                    return Ok(0);
                }
                _ => unreachable!(),
            },
        }
    }

    let args = args.split_off(opts.index());
    if args.is_empty() {
        eprintln!("{}", usage_line());
        return Ok(1);
    }

    for arg in args {
        purge::purge(&arg)?;
    }

    Ok(0)
}

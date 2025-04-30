use regex::engine::parser::parse;
use std::{env::args, io};

fn main() -> io::Result<()> {
    let args: Vec<String> = args().collect();

    if let Some(expr) = args.get(1) {
        let ast = parse(expr).map_err(|msg| io::Error::new(io::ErrorKind::InvalidInput, msg))?;
        // println!("{:?}", ast);
        println!("{}", ast);
    } else {
        eprintln!("{}", "could not get expr");
    }

    Ok(())
}

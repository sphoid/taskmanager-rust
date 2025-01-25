use std::process;

mod project;
mod cli;

fn main() {
	// let args = Cli::parse();

	// let mut rtc = cli::RuntimeConfig::build(args.namespace, args.command, args._args).unwrap_or_else(|err| {
	let mut rtc = cli::RuntimeConfig::build().unwrap_or_else(|err| {
		println!("Runtime config error: {err}");
		process::exit(1);
	});

	rtc.run().unwrap_or_else(|err| {
        println!("Runtime error: {err}");
        process::exit(1);
    });


	// println!("{rt:?}");
}

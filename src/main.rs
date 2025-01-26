use std::process;

mod config;
mod project;
mod cli;

fn main() {
	let mut rtc = cli::RuntimeConfig::build().unwrap_or_else(|err| {
		println!("Runtime config error: {err}");
		process::exit(1);
	});

	rtc.run().unwrap_or_else(|err| {
        println!("Runtime error: {err}");
        process::exit(1);
    });
}

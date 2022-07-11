extern crate getopts;
use getopts::Options;
use std::env::args;
use std::fs::File;
use std::path::Path;
use std::process;
use std::str::FromStr;

fn main() {
	let args: Vec<_> = args().collect();
	let program = args[0].clone();

	let mut opts = Options::new();
	opts.optflag("h", "help", "print this help menu");
	opts.optopt(
		"",
		"supplementary",
		"reads with supplementary alignments",
		"sel|del",
	);

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => {
			eprintln!("[ERROR] {}", f.to_string());
			print_usage(&program, opts);
			process::exit(1);
		}
	};

	if matches.opt_present("h") {
		print_usage(&program, opts);
		return;
	}

	let mut param: FilterParam = Default::default();

	param.supplementary = match matches.opt_get_default("supplementary", Mode::No) {
		Ok(m) => m,
		Err(err) => {
			eprintln!("[ERROR] {}: 'supplementary'", err);
			process::exit(1);
		}
	};

	if matches.free.len() != 1 {
		print_usage(&program, opts);
		process::exit(1);
	}

	let path = Path::new(&matches.free[0]);
	let file = match File::open(&path) {
		Ok(file) => file,
		Err(err) => {
			eprintln!("[ERROR] open: {} {}", path.display(), err);
			process::exit(1);
		}
	};
}

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} [options] <SAM file>", program);
	print!("{}", opts.usage(&brief));
}

#[derive(Clone, Copy, Default)]
enum Mode {
	#[default]
	No,
	Sel,
	Del,
}

impl FromStr for Mode {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s == "sel" {
			Ok(Mode::Sel)
		} else if s == "del" {
			Ok(Mode::Del)
		} else {
			Err("unrecognized mode".to_string())
		}
	}
}

#[derive(Clone, Copy, Default)]
struct FilterParam {
	supplementary: Mode,
}

fn filter(input: &File, output: &File, param: FilterParam) {}

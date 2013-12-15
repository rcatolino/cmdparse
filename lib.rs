#[desc = "Library to parse simple command line arguments"];
#[license = "MIT"];

// argument : string passed by the user via the command line
// command : kind of argument that is unique, doesn't start with '-',
// option : kind of argument that starts with '-' or '--', has an optional value.
// value : kind argument that is anonymous and has a value.
//         Can only be last or followed by other values.
use std::hashmap::HashMap;
use std::result::Result;
use std::rc::Rc;

static min_align: uint = 11;
pub mod Flags {
  pub static Defaults: uint = 0;
  pub static RejectMultiple: uint = 1 << 0;
  pub static Hidden: uint = 1 << 1;
  pub static TakesArg: uint = 1 << 2;
  pub static TakesOptionalArg: uint = 1 << 3;
}

pub struct Cmd {
  name: &'static str,
  description: Option<&'static str>,
  options: ~[Rc<Opt>]
}

priv struct Res {
  time_passed: uint,   // Number of time we've seen this option
  values: ~[~str],     // Values of the arguments it's been given
}

priv struct RawArg {
  option: bool,       // Options start with - or --
  value: ~str,
}

impl RawArg {
  pub fn new(value: ~str, option: bool) -> RawArg {
    RawArg { option: option, value: value }
  }
}

pub struct Opt {
  long_name: Option<&'static str>,
  short_name: Option<&'static str>,
  description: Option<&'static str>,
  flags: uint,
  result_idx: uint
}

impl Opt {
  fn new(long_name: Option<&'static str>,
         short_name: Option<&'static str>,
         descr: Option<&'static str>,
         flags: uint,
         res_idx: uint) -> Opt {

    Opt { long_name: long_name, short_name: short_name, description: descr,
          flags: flags, result_idx: res_idx }
  }

  fn has_flags(&self, flags: uint) -> bool {
    self.flags & flags == flags
  }
}

pub struct OptContext {
  // A summary describing the application and/or an exemple.
  summary: &'static str,
  // A map of globally valid options.
  options: HashMap<&'static str, Rc<Opt>>,
  // A list of valid commands.
  commands: ~[~Cmd],
  // The arguments provided by the user.
  raw_args: ~[RawArg],
  // The results found for each Opt after validation
  results: ~[Res],
  // Align
  alignment: uint,
}

impl OptContext {

  pub fn new(description: &'static str, args: ~[~str]) -> OptContext {
    OptContext {
      summary: description,
      options: HashMap::new(),   // Valid options
      commands: ~[],  // Valid commands
      raw_args: OptContext::prep_args(args),
      results: ~[],
      alignment: min_align, // Minimum aligment
    }
  }

  fn prep_args(args: ~[~str]) -> ~[RawArg] {
    let mut vect = ~[];

    // skip the program name
    for arg in args.move_iter().skip(1) {
      // Check if this first character is '-'
      let (name, option) =
        if (arg[0] == '-' as u8) {
          // this is an option
          (if (arg[1] == '-' as u8) {
            arg.slice_from(2).to_owned()
          } else {
            arg.slice_from(1).to_owned()
          },
          true)
        } else {
          (arg, false)
        };
      vect.push(RawArg::new(name, option));
    }
    vect
  }

  pub fn add_option(&mut self,
                    long_name: Option<&'static str>,
                    short_name: Option<&'static str>,
                    description: Option<&'static str>,
                    flags: uint) -> Result<Rc<Opt>, &'static str> {

    let opt = Rc::new(Opt::new(long_name, short_name, description, flags,
                               self.results.len()));
    self.results.push(Res { time_passed:0, values:~[] });
    match long_name {
      Some(name) => {
        // Update the alignment and check that there is a name.
        self.alignment = std::cmp::max(self.alignment, name.len() + min_align);
        if name.len() < 2 {
          return Err("A long name needs more than 1 character");
        } else if !self.options.insert(name, opt.clone()) {
          return Err("An option with the same long name was already added");
        }
      }
      None => if short_name.is_none() {
        return Err("An option needs either a short or a long name")
      }
    }

    match short_name {
      Some(name) => if name.len() > 1 {
        return Err("A short name can have only 1 character");
      } else if !self.options.insert(name, opt.clone()) {
        return Err("An option with the same short name was already added");
      },
      None => {}
    }

    Ok(opt)
  }

  pub fn validate<'a>(&mut self) -> Result<(), &'static str> {
    for arg in self.raw_args.iter() {
      if arg.option {
        match self.options.find_equiv(&arg.value) {
          Some(opt) => {}, None => {}
        }
      }
    }

    Err("Unimplemented")
  }

  pub fn count(&self, opt: Rc<Opt>) -> uint {
    0
  }

  pub fn check(&self, opt: Rc<Opt>) -> bool {
    false
  }

  pub fn take_values<T: FromStr>(&mut self, opt: Rc<Opt>) -> Either<~[T], uint> {
    Right(self.count(opt))
  }

  pub fn take_value<T: FromStr>(&mut self, opt: Rc<Opt>) -> Either<T, bool> {
    Right(self.check(opt))
  }

  pub fn print_help(&self, msg: Option<&str>) {
    match msg {
      Some(err) => println!("Error : {:s}", err), None => {}
    }
    print("Usage: \n  ");
    println(self.summary);
    println("Valid options :");
    self.options.each_value(|opt| self.print_opt(opt.borrow()));
  }

  fn print_opt(&self, opt: &Opt) -> bool {
    if opt.has_flags(Flags::Hidden) {
      return true;
    }
    print("  ");
    match opt.short_name {
      Some(name) => print!("-{:s}", name),
      None => print("  ")
    }
    match opt.long_name {
      Some(value) => {
        let mut align = self.alignment - value.len();
        if opt.short_name.is_some() {
          print(",");
        }
        print!("\t--{:s}", value);
        if opt.has_flags(Flags::TakesOptionalArg) {
          print!("[=argument]");
          align -= 11;
        } else if opt.has_flags(Flags::TakesArg) {
          print!("=argument");
          align -= 9;
        }
        print!("{:s}\t", " ".repeat(align));
      }
      None => {
        let mut align = self.alignment;
        if opt.has_flags(Flags::TakesOptionalArg) {
          print!(" [argument]");
          align -= 11;
        } else if opt.has_flags(Flags::TakesArg) {
          print!(" argument");
          align -= 9;
        }
        print!(",\t  {:s}\t", " ".repeat(align));
      }
    }
    match opt.description {
      Some(value) => println(value),
      None => println("")
    }
    true
  }

}

#[desc = "Library to parse simple command line arguments"];
#[license = "MIT"];
#[link(name="cmdparse")];

/*!
  Command line option parsing

  # Features
  - Definition of option with short and/or long names.
  - Options taking optional or mandatory arguments.
  - Automatic help message generation.

  # To do
  - Commands taking their own options
  - Anonymous arguments

  # Example, to parse the options :
  "-h/--help, -l, --option, -a [optional_argument(int)], -m <mandatory_argument(str)>"

  ```rust
  // First create the context with the program summary and the input arguments :
  let mut ctx = Context::new("example [options]", os::args());

  // Then add the authorized options.
  let help_opt = ctx.add_option(Some("help"), Some("h"), Some("Display this help"),
                                Flags::Defaults).unwrap();
  let o_opt = ctx.add_option(None, Some("l"), Some("Activate the option l"),
                             Flags::Defaults).unwrap();
  let l_opt = ctx.add_option(Some("option"), None, Some("Activate some option"),
                             Flags::Defaults).unwrap();
  let a_opt = ctx.add_option(None, Some("a"), Some("Activate the option a"),
                             Flags::TakesOptionalArg).unwrap();
  let m_opt = ctx.add_option(None, Some("m"), Some("Activate the option m"),
                             Flags::TakesArg).unwrap();
  // add_option() can only return None if the option was specified in
  // a way that makes no sense, eg no long name and no short name.
  // You probably want to fail in this case, hence the unwrap().

  // Validate the input arguments against the valid options
  match ctx.validate() {
    Err(msg) => {
      // The input options didn't match the authorized ones. Display help.
      ctx.print_help(Some(msg.as_slice()));
      return;
    }
    Ok(()) => {}
  }

  // Do stuff with the results
  if ctx.check(help_opt) {
    ctx.print_help(None);
    return;
  }

  let a_value = match ctx.take_value(a_opt) {
    Left(Some(some_int)) => println!("a : {:d}", some_int),
    Left(None) => println("a : the argument should be an int!!!"),
    Right(passed) => if passed {
      println("the option 'a' was passed without an argument.");
    } else {
      println("the option 'a' was not passed.");
    }
  };

  // etc.
  ```

*/

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
  pub static Unique: uint = 1 << 0;
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
  passed: uint,   // Number of time we've seen this option
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

  fn has_flag(&self, flags: uint) -> bool {
    (self.flags & flags) != 0
  }
}

pub struct Context {
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

impl Context {

  pub fn new(description: &'static str, args: ~[~str]) -> Context {
    Context {
      summary: description,
      options: HashMap::new(),   // Valid options
      commands: ~[],  // Valid commands
      raw_args: Context::prep_args(args),
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
    self.results.push(Res { passed:0, values:~[] });
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

  fn check_next_value(&self) -> bool {
    self.raw_args.head_opt().map_default(false, |narg| if !narg.option {
      true
    } else {
      false
    })
  }

  pub fn validate(&mut self) -> Result<(), ~str> {
    // Peekable iterator not really usable here since it prevents
    // from mutating the rest of the elements while borrowed.
    let mut oarg = self.raw_args.shift_opt();
    while oarg.is_some() {
      let arg = oarg.unwrap(); // Can't fail since it's some.
      if !arg.option {
        return Err(format!("Unexpected argument : {:s}", arg.value));
      }

      match self.options.find_equiv(&arg.value) {
        Some(opt) => {
          let idx = opt.borrow().result_idx;
          self.results[idx].passed += 1;
          let res = &self.results[idx];
          if res.passed > 1 && opt.borrow().has_flag(Flags::Unique) {
            return Err(format!("The option : {:s} was given more than once",
                               arg.value));
          } else if opt.borrow().has_flag(Flags::TakesArg) {
              if self.check_next_value() {
                Some((self.raw_args.shift().value, idx))
              } else {
                return Err(format!("Missing argument for option : {:s}",
                                   arg.value));
              }
          } else if opt.borrow().has_flag(Flags::TakesOptionalArg)
                    && self.check_next_value() {
              Some((self.raw_args.shift().value, idx))
          } else {
            None
          }
        },
        None => return Err(format!("Invalid option : {:s}", arg.value)),
      }.map(|(value, idx)| self.results[idx].values.push(value));

      oarg = self.raw_args.shift_opt();
    }

    Ok(())
  }

  pub fn count(&self, opt: Rc<Opt>) -> uint {
    match self.results.get_opt(opt.borrow().result_idx) {
      Some(res) => res.passed,
      None => 0
    }
  }

  pub fn check(&self, opt: Rc<Opt>) -> bool {
    self.count(opt) != 0
  }

  pub fn take_values<T: FromStr>(&mut self, opt: Rc<Opt>) -> Either<~[T], uint> {
    Right(self.count(opt))
  }

  pub fn take_value<T: FromStr>(&mut self, opt: Rc<Opt>) -> Either<Option<T>, bool> {
    match self.results.get_opt(opt.borrow().result_idx) {
      Some(res) => match res.values.head_opt() {
        Some(value) => Left(from_str(*value)),
        None => if res.passed == 0 {
          Right(false)
        } else {
          Right(true)
        }
      },
      None => Right(false),
    }
  }

  pub fn print_help(&self, msg: Option<&str>) {
    let mut printed: ~[bool] = ~[];
    printed.grow_fn(self.results.len(), |_| false);
    match msg {
      Some(err) => println!("Error : {:s}", err), None => {}
    }
    print("Usage: \n  ");
    println(self.summary);
    println("Valid options :");
    self.options.each_value(|opt| if !printed[opt.borrow().result_idx] {
      printed[opt.borrow().result_idx] = true;
      self.print_opt(opt.borrow())
    } else {
      true
    });
  }

  fn print_opt(&self, opt: &Opt) -> bool {
    if opt.has_flag(Flags::Hidden) {
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
        if opt.has_flag(Flags::TakesOptionalArg) {
          print!("[=argument]");
          align -= 11;
        } else if opt.has_flag(Flags::TakesArg) {
          print!("=argument");
          align -= 9;
        }
        print!("{:s}\t", " ".repeat(align));
      }
      None => {
        let mut align = self.alignment;
        if opt.has_flag(Flags::TakesOptionalArg) {
          print!(" [argument]");
          align -= 11;
        } else if opt.has_flag(Flags::TakesArg) {
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

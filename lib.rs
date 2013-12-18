#[desc = "Library to parse simple command line options"];
#[license = "MIT"];
#[link(name="cmdparse")];

/*!
  Command line option parsing

  # Features
  - Definition of option with short and/or long names.
  - Options taking optional or mandatory arguments.
  - Grouping of short options
  - Automatic help message generation.

  # To do
  - Commands taking their own options

  # Example, to parse the options :
  "-h/--help, -l, --option, -a [optional_argument(int)], -m mandatory_argument(str) leftover_argument"

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

static min_align: uint = 15;
pub mod Flags {
  pub static Defaults: uint = 0;
  pub static Unique: uint = 1 << 0;
  pub static Hidden: uint = 1 << 1;
  pub static TakesArg: uint = 1 << 2;
  pub static TakesOptionalArg: uint = 1 << 3;
}

priv struct Res {
  passed: uint,        // Number of time we've seen this option
  values: ~[~str],     // Arguments it's been given
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

priv struct Opt {
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
  alignment: uint,
  // A map of globally valid options.
  options: HashMap<&'static str, Rc<Opt>>,
  // The arguments provided by the user.
  raw_args: ~[RawArg],
  // The results found for each Opt after validation
  results: ~[Res],
  // The arguments left after validation
  residual_args: ~[~str],
}

impl Context {

  pub fn new(description: &'static str, args: ~[~str]) -> Context {
    Context {
      summary: description,
      alignment: min_align,
      options: HashMap::new(),
      raw_args: Context::prep_args(args),
      results: ~[],
      residual_args: ~[],
    }
  }

  fn prep_args(args: ~[~str]) -> ~[RawArg] {
    let mut vect = ~[];

    // skip the program name
    for arg in args.move_iter().skip(1) {
      if arg.starts_with("--") {
        // Long option
        let mut cit = arg.slice_from(2).splitn('=', 1);
        cit.next().and_then(|ovalue| {
          vect.push(RawArg::new(ovalue.to_owned(), true));
          cit.next()
        }).map(|ovalue| vect.push(RawArg::new(ovalue.to_owned(), false)));
      } else if arg.starts_with("-") {
        // Short option(s)
        for c in arg.chars().skip(1) {
          vect.push(RawArg::new(c.to_str(), true));
        }
      } else {
        vect.push(RawArg::new(arg, false));
      }
    }
    vect
  }

  /// Specify valid options for your program. Return Err() if
  /// the option has neither short nor long name or if an option
  /// with the same name was already added.
  // TODO: change the type of short name to Option<char>
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
        // The alignment is used in print_help() to make sure the columns are
        // aligned.
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

  /// Validate the input arguments against the options specified via add_option()
  /// Return an Err() when the input isn't valid.
  pub fn validate(&mut self) -> Result<(), ~str> {
    // Peekable iterator not really usable here since it prevents
    // from mutating the rest of the elements while borrowed.
    let mut oarg = self.raw_args.shift_opt();
    while oarg.is_some() {
      let arg = oarg.unwrap(); // Can't fail since it's some.
      if !arg.option {
        self.residual_args.push(arg.value);
      } else {
        match self.options.find_equiv(&arg.value) {
          Some(opt) => {
            if self.residual_args.len() != 0 {
              return Err(format!("Unexpected argument : {:s}.",
                                 self.residual_args.shift()));
            }
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
      }

      oarg = self.raw_args.shift_opt();
    }

    Ok(())
  }

  /// Return whether the option was given among the input arguments.
  pub fn check(&self, opt: Rc<Opt>) -> bool {
    self.count(opt) != 0
  }

  /// Returns the value attached with the given option. (ie --option=value)
  /// If the value is cannot be parsed into a valid T, returns Left(None),
  /// If the option was given with no value, returns Right(true),
  /// or Right(false) if the option wasn't given.
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

  /// Get an array containing the residual arguments.
  pub fn get_args<'a>(&'a self) -> &'a[~str] {
    self.residual_args.as_slice()
  }

  /// Variant of check() for when the option could be specified an
  /// arbitrary number of times. (eg -vvv for the verbosity level)
  pub fn count(&self, opt: Rc<Opt>) -> uint {
    match self.results.get_opt(opt.borrow().result_idx) {
      Some(res) => res.passed,
      None => 0
    }
  }

  /// Variant of take_value() for when the option can receive several values.
  /// eg --output=file1 --output=pipe1
  pub fn take_values<T: FromStr>(&mut self, opt: Rc<Opt>) -> Either<~[Option<T>], uint> {
    match self.results.get_opt(opt.borrow().result_idx) {
      Some(res) => if res.values.len() == 0 {
        Right(res.passed)
      } else {
        Left(res.values.map(|value| from_str(*value)))
      },
      None => Right(0),
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
    // Not using tabs cause they mess with the alignment
    if opt.has_flag(Flags::Hidden) {
      return true;
    }
    print("  ");
    // Print until the long option
    let mut align = self.alignment;
    match opt.short_name {
      Some(name) => {
        print!("-{:s}", name);
        if opt.long_name.is_none() {
          if opt.has_flag(Flags::TakesOptionalArg) {
            print!(" [argument]");
            align -= 11;
          } else if opt.has_flag(Flags::TakesArg) {
            print!(" argument");
            align -= 9;
          }
        }
        print(",     ");
      }
      None => print("        ")
    }
    // Print until the description
    match opt.long_name {
      Some(value) => {
        align -= value.len()+2;
        print!("--{:s}", value);
        if opt.has_flag(Flags::TakesOptionalArg) {
          print!("[=argument]");
          align -= 11;
        } else if opt.has_flag(Flags::TakesArg) {
          print!("=argument");
          align -= 9;
        }
      }
      None => {}
    }
    print!("{:s}  ", " ".repeat(align));
    // Print until the end
    match opt.description {
      Some(value) => println(value),
      None => print("\n")
    }
    true
  }
}

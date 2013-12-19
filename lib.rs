#[desc = "Library to parse simple command line options"];
#[license = "MIT"];

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

priv enum RawArg {
  Short(char),
  Long(~str),
  Neither(~str),
}

impl RawArg {
  pub fn option(&self) -> bool {
    match *self {
      Neither(_) => false, _ => true
    }
  }

  pub fn value(self) -> ~str {
    match self {
      Short(c) => c.to_str(), Long(a) | Neither(a) => a,
    }
  }
}

trait Validable {
  fn validate(&self, name: ~str, rargs: &mut ~[RawArg], results: &mut [Res]) ->
    Result<(), ~str>;
}

priv struct Opt {
  short_name: Option<char>,
  long_name: Option<&'static str>,
  description: Option<&'static str>,
  flags: uint,
  result_idx: uint
}

impl Opt {
  fn new(long_name: Option<&'static str>,
         short_name: Option<char>,
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

impl Validable for Opt {
  fn validate(&self, opt_name: ~str, rargs: &mut ~[RawArg], results: &mut [Res]) ->
    Result<(), ~str> {

    let res = &mut results[self.result_idx];
    res.passed += 1;
    if res.passed > 1 && self.has_flag(Flags::Unique) {
      return Err(format!("The option : {:s} was given more than once", opt_name));
    } else if self.has_flag(Flags::TakesArg | Flags::TakesOptionalArg) {
      if rargs.head_opt().map_default(false, |narg| !narg.option()) {
        Some(rargs.shift().value())
      } else if self.has_flag(Flags::TakesArg) {
        return Err(format!("Missing argument for option : {:s}", opt_name));
      } else {
        None
      }
    } else {
      None
    }.map(|value| res.values.push(value));

    Ok(())
  }
}

priv struct Cmd; /* {
  name: &'static str,
  description: &'static str,
  loptions: HashMap<&'static str, Rc<Opt>>,
  soptions: HashMap<char, Rc<Opt>>,
  results: ~[Res],
  flags: uint,
  result_idx: uint
}

impl Validable for Cmd {
  /// Validate the input arguments against the options specified via add_option().
  /// Return an Err() when the input isn't valid.
  fn validate(&self, cmd_name: ~str, rargs: &mut ~[RawArg],
              parent_results: &mut [Res]) -> Result<(), ~str> {
    // First check that the command has only been given once
    if results[self.result_idx] > 0 {
      return Err(~"Unexpected command : {:s}", cmd_name);
    } else {
      result[self.result_idx] += 1;
    }

    while rargs.len() > 0 {
      let raw_arg = rargs.shift(); // Can't fail since len() > 0;
      match match raw_arg {
        Short(sname) => (self.soptions.find(&sname).
                         map(|opt| opt.borrow() as &Validable), sname.to_str(), true),
        Long(lname) => (self.loptions.find_equiv(&lname.as_slice()).
                        map(|opt| opt.borrow() as &Validable), lname, true),
        Neither(name) => (None, name, false),
      } {
        (None, name, true) => return Err(format!("Invalid option : {:s} for command : {:s}.",
                                                 name, cmd_name)),
        (None, name, false) => self.residual_args.push(name),
        (Some(to_validate), name, _) => {
          match to_validate.validate(name, rargs, self.results) {
            Err(msg) => return Err(msg),
            Ok(_) => if self.residual_args.len() != 0 {
              return Err(format!("Unexpected argument : {:s}.", self.residual_args.shift()))
            }
          }
        }
      }
    }
    Ok(())
  }
}
*/

impl Validable for Cmd {
  fn validate(&self, opt_name: ~str, rargs: &mut ~[RawArg], results: &mut [Res]) ->
    Result<(), ~str> {
    Err(~"Unimplemented")
  }
}

pub struct LocalContext {
  priv alignment: uint,
  priv description: &'static str,
  // Maps of locally valid options short/long.
  priv loptions: HashMap<&'static str, Rc<Opt>>,
  priv soptions: HashMap<char, Rc<Opt>>,
  // The results found for each Opt after validation
  priv results: ~[Res],
}

impl LocalContext {
  pub fn new(description: &'static str) -> LocalContext {
    LocalContext {
      alignment: min_align,
      description: description,
      loptions: HashMap::new(),
      soptions: HashMap::new(),
      results: ~[],
    }
  }

  fn parse(&mut self, commands: Option<&HashMap<&'static str, Cmd>>,
           rargs: &mut ~[RawArg], residual_args: &mut ~[~str]) -> Result<(), ~str> {
    while rargs.len() > 0 {
      let raw_arg = rargs.shift(); // Can't fail since len() > 0;
      match match raw_arg {
        Short(sname) => (self.soptions.find(&sname).
                         map(|opt| opt.borrow() as &Validable), sname.to_str(), true),
        Long(lname) => (self.loptions.find_equiv(&lname.as_slice()).
                        map(|opt| opt.borrow() as &Validable), lname, true),
        Neither(name) => (commands.and_then(|cmds| cmds.find_equiv(&name.as_slice()).
                          map(|cmd| cmd as &Validable)), name, false),
      } {
        (None, name, true) => return Err(format!("Invalid option : {:s}.", name)),
        (None, name, false) => residual_args.push(name),
        (Some(to_validate), name, _) => {
          match to_validate.validate(name, rargs, self.results) {
            Err(msg) => return Err(msg),
            Ok(_) => if residual_args.len() != 0 {
              return Err(format!("Unexpected argument : {:s}.", residual_args.shift()))
            }
          }
        }
      }
    }
    Ok(())
  }
}

impl OptGroup for LocalContext {
  /// Specify valid options for your program. Return Err() if
  /// the option has neither short nor long name or if an option
  /// with the same name was already added.
  fn add_option(&mut self, long_name: Option<&'static str>,
                short_name: Option<char>, description: Option<&'static str>,
                flags: uint) -> Result<Rc<Opt>, &'static str> {

    let opt = Rc::new(Opt::new(long_name, short_name, description, flags,
                               self.results.len()));
    self.results.push(Res { passed:0, values:~[] });
    match long_name {
      Some(name) => {
        // The alignment is used in print_help() to make sure the columns are
        // aligned.
        self.alignment = std::cmp::max(self.alignment, name.len() + min_align);
        if !self.loptions.insert(name, opt.clone()) {
          return Err("An option with the same long name was already added");
        }
      }
      None => if short_name.is_none() {
        return Err("An option needs either a short or a long name")
      }
    }

    match short_name {
      Some(name) => if !self.soptions.insert(name, opt.clone()) {
        return Err("An option with the same short name was already added");
      },
      None => {}
    }

    Ok(opt)
  }

  /// Return whether the option was given among the input arguments.
  fn check(&self, opt: Rc<Opt>) -> bool {
    self.count(opt) != 0
  }

  /// Returns the value attached with the given option. (ie --option=value).
  /// If the value is cannot be parsed into a valid T, returns Left(None).
  /// If the option was given with no value returns Right(true),
  /// or Right(false) if the option wasn't given.
  fn take_value<T: FromStr>(&mut self, opt: Rc<Opt>) -> Either<Option<T>, bool> {
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

  /// Variant of check() for when the option could be specified an
  /// arbitrary number of times. (eg -vvv for the verbosity level)
  fn count(&self, opt: Rc<Opt>) -> uint {
    match self.results.get_opt(opt.borrow().result_idx) {
      Some(res) => res.passed,
      None => 0
    }
  }

  /// Variant of take_value() for when the option can receive several values.
  /// eg --output=file1 --output=pipe1
  fn take_values<T: FromStr>(&mut self, opt: Rc<Opt>) -> Either<~[Option<T>], uint> {
    match self.results.get_opt(opt.borrow().result_idx) {
      Some(res) => if res.values.len() == 0 {
        Right(res.passed)
      } else {
        Left(res.values.map(|value| from_str(*value)))
      },
      None => Right(0),
    }
  }
}

pub struct Context {
  // A summary describing the application and/or an exemple.
  priv commands: HashMap<&'static str, Cmd>,
  // The arguments provided by the user.
  priv raw_args: ~[RawArg],
  // The arguments left after validation
  priv residual_args: ~[~str],
  // The context containing all the global options.
  priv inner_ctx: LocalContext,
}

pub trait OptGroup {
  fn check(&self, opt: Rc<Opt>) -> bool;
  fn take_value<T: FromStr>(&mut self, opt: Rc<Opt>) -> Either<Option<T>, bool>;
  fn count(&self, opt: Rc<Opt>) -> uint;
  fn take_values<T: FromStr>(&mut self, opt: Rc<Opt>) -> Either<~[Option<T>], uint>;
  fn add_option(&mut self, lname: Option<&'static str>,
                sname: Option<char>, description: Option<&'static str>,
                flags: uint) -> Result<Rc<Opt>, &'static str>;
}

impl Context {
  pub fn new(description: &'static str, args: ~[~str]) -> Context {
    Context {
      commands: HashMap::new(),
      raw_args: Context::prep_args(args),
      residual_args: ~[],
      inner_ctx: LocalContext::new(description),
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
          vect.push(Long(ovalue.to_owned()));
          cit.next()
        }).map(|ovalue| vect.push(Neither(ovalue.to_owned())));
      } else if arg.starts_with("-") {
        // Short option(s)
        for c in arg.chars().skip(1) {
          vect.push(Short(c));
        }
      } else {
        vect.push(Neither(arg));
      }
    }

    vect
  }

  /// Validate the input arguments against the options specified via add_option().
  /// Return an Err() when the input isn't valid.
  pub fn validate(&mut self) -> Result<(), ~str> {
    self.inner_ctx.parse(Some(&self.commands), &mut self.raw_args, &mut self.residual_args)
  }

  /// Get an array containing the residual arguments.
  pub fn get_args<'a>(&'a self) -> &'a[~str] {
    self.residual_args.as_slice()
  }

  pub fn print_help(&self, msg: Option<&str>) {
    let mut printed: ~[bool] = ~[];
    printed.grow_fn(self.inner_ctx.results.len(), |_| false);
    match msg {
      Some(err) => println!("Error : {:s}", err), None => {}
    }
    print("Usage: \n  ");
    println(self.inner_ctx.description);
    println("Valid options :");
    for opt in self.inner_ctx.soptions.iter().map(|(_, a)| a).
      chain(self.inner_ctx.loptions.iter().map(|(_,a)| a)) {
      if !opt.borrow().has_flag(Flags::Hidden) && !printed[opt.borrow().result_idx] {
        printed[opt.borrow().result_idx] = true;
        self.print_opt(opt.borrow())
      }
    }
  }

  fn print_opt(&self, opt: &Opt) {
    // Not using tabs cause they mess with the alignment
    print("  ");
    // Print until the long option
    let mut align = self.inner_ctx.alignment;
    match opt.short_name {
      Some(name) => {
        print!("-{:s}", name.to_str());
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
  }
}

impl OptGroup for Context {
  fn add_option(&mut self, lname: Option<&'static str>,
                sname: Option<char>, description: Option<&'static str>,
                flags: uint) -> Result<Rc<Opt>, &'static str> {
    self.inner_ctx.add_option(lname, sname, description, flags)
  }
  // TODO add checks
  fn check(&self, opt: Rc<Opt>) -> bool {
    self.inner_ctx.check(opt)
  }

  fn take_value<T: FromStr>(&mut self, opt: Rc<Opt>) -> Either<Option<T>, bool> {
    self.inner_ctx.take_value(opt)
  }

  fn count(&self, opt: Rc<Opt>) -> uint {
    self.inner_ctx.count(opt)
  }

  fn take_values<T: FromStr>(&mut self, opt: Rc<Opt>) -> Either<~[Option<T>], uint> {
    self.inner_ctx.take_values(opt)
  }
}

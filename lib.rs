#[pkgid = "cmdparse"];
#[desc = "Library to parse simple command line options"];
#[license = "MIT"];

/*!
  Command line option parsing

  # Features
  - Definition of option with short and/or long names.
  - Options taking optional or mandatory arguments.
  - Grouping of short options
  - Automatic help message generation.
  - Commands taking their own options

  # Example, to parse the options :
  "-h/--help, -l, --option, -a [optional_argument(int)], -m mandatory_argument(str) leftover_argument"

  ```rust
  // First create the context with the program summary and the input arguments :
  let mut ctx = Context::new("cmdparse [options]", os::args());

  // Then add the authorized options.
  // Use the convenience wrappers :
  let help_opt = ctx.add_opt("help", 'h', "Display this help");
  let o_opt = ctx.add_sopt('l', "Activate the option l");
  let l_opt = ctx.add_lopt("option", "Activate some option");

  // Use the full add_option function :
  let a_opt = ctx.add_option(None, Some('a'), Some("Activate the option a"),
                             Flags::TakesOptionalArg).unwrap();
  let m_opt = ctx.add_option(None, Some('m'), Some("Activate the option m"),
                             Flags::TakesArg).unwrap();
  let m_opt = ctx.add_option(Some("long"), None, Some("Activate the option n"),
                             Flags::TakesOptionalArg).unwrap();
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
  if help_opt.check() {
    ctx.print_help(None);
    return;
  }

  match a_opt.take_value::<int>() {
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

use std::cast::transmute;
use std::cell::RefCell;
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

#[deriving(Clone)]
pub struct Opt {
  priv short_name: Option<char>,
  priv long_name: Option<&'static str>,
  priv description: Option<&'static str>,
  priv flags: uint,
  priv result: Rc<RefCell<Res>>,
}

impl Opt {
  fn new(long_name: Option<&'static str>,
         short_name: Option<char>,
         descr: Option<&'static str>,
         flags: uint,
         result: Rc<RefCell<Res>>) -> Opt {

    Opt { long_name: long_name, short_name: short_name, description: descr,
          flags: flags, result: result }
  }

  fn has_flag(&self, flags: uint) -> bool {
    (self.flags & flags) != 0
  }

  fn validate(&self, opt_name: ~str, rargs: &mut ~[RawArg],
              residual_args: &mut ~[~str]) -> Result<(), ~str> {

    let mut res = self.result.borrow().borrow_mut();
    res.get().passed += 1;
    if residual_args.len() != 0 {
      return Err(format!("Unexpected argument : {:s}.", residual_args.shift()))
    } else if res.get().passed > 1 && self.has_flag(Flags::Unique) {
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
    }.map(|value| res.get().values.push(value));

    Ok(())
  }

  /// Return whether the option was given among the input arguments.
  pub fn check(&self) -> bool {
    self.count() != 0
  }

  /// Returns the value attached with the given option. (ie --option=value).
  /// If the value is cannot be parsed into a valid T, returns Left(None).
  /// If the option was given with no value returns Right(true),
  /// or Right(false) if the option wasn't given.
  pub fn take_value<T: FromStr>(&self) -> Either<Option<T>, bool> {
    let mut res = self.result.borrow().borrow_mut();
    let passed = res.get().passed;
    match res.get().values.shift_opt() {
      // Is there a way to avoid allocation of a new string when T: Str ?
      Some(value) => Left(from_str(value)),
      None => if passed == 0 {
        Right(false)
      } else {
        Right(true)
      }
    }
  }

  /// Variant of check() for when the option could be specified an
  /// arbitrary number of times. (eg -vvv for the verbosity level)
  pub fn count(&self) -> uint {
    self.result.borrow().borrow().get().passed
  }

  /// Variant of take_value() for when the option can receive several values.
  /// eg --output=file1 --output=pipe1
  pub fn take_values<T: FromStr>(&self) -> Either<~[Option<T>], uint> {
    let mut res = self.result.borrow().borrow_mut();
    if res.get().values.len() == 0 {
      Right(res.get().passed)
    } else {
      Left(res.get().values.map(|value| from_str(*value)))
    }
  }
}

#[deriving(Clone)]
pub struct CmdRes(Rc<RefCell<bool>>);

impl CmdRes {
  pub fn check(&self) -> bool {
    let res = (*self).borrow().borrow();
    *res.get()
  }

  fn set(&self) {
    let mut res = (*self).borrow().borrow_mut();
    *res.get() = true;
  }
}

pub struct Cmd {
  priv inner_ctx: LocalContext,
  priv result: CmdRes,
}

impl Cmd {
  fn new(description: &'static str) -> Cmd {
    Cmd { inner_ctx: LocalContext::new(description),
          result: CmdRes(Rc::from_mut(RefCell::new(false))) }
  }

  fn validate(&mut self, cmd_name: ~str, rargs: &mut ~[RawArg],
              residual_args: &mut ~[~str]) -> Result<(), ~str> {
    // First check that the command has only been given once
    if residual_args.len() != 0 {
      Err(format!("Unexpected argument : {:s}.", residual_args.shift()))
    } else if self.result.check() {
      Err(format!("Unexpected command : {:s}", cmd_name))
    } else {
      self.result.set();
      self.inner_ctx.parse(&mut HashMap::new(), rargs, residual_args)
    }
  }
}

struct LocalContext {
  alignment: uint,
  // A summary describing the application and/or an exemple.
  description: &'static str,
  // Maps of locally valid options short/long.
  loptions: HashMap<&'static str, Opt>,
  soptions: HashMap<char, Opt>,
  // List of options added. Needed for print_help
  print_options: ~[Opt],
}

impl LocalContext {
  pub fn new(description: &'static str) -> LocalContext {
    LocalContext {
      alignment: min_align,
      description: description,
      loptions: HashMap::new(),
      soptions: HashMap::new(),
      print_options: ~[],
    }
  }

  fn parse(&mut self, cmds: &mut HashMap<&'static str, Cmd>,
           rargs: &mut ~[RawArg], residual_args: &mut ~[~str]) -> Result<(), ~str> {
    while rargs.len() > 0 {
      let raw_arg = rargs.shift(); // Can't fail since len() > 0;
      match match match raw_arg {
        Short(sname) => (Left(self.soptions.find(&sname)), sname.to_str()),
        Long(lname) => (Left(self.loptions.find_equiv(&lname.as_slice())), lname),
        Neither(nname) => (Right(unsafe {
          // FIXME: replace transmute with find_mut_equiv or
          // equivalent once it is added to libstd
          cmds.find_mut(&transmute(nname.as_slice()))
        }), nname),
      } {
        (Left(None), name) => Err(format!("Invalid option : {:s}.", name)),
        (Right(None), name) => { residual_args.push(name); Ok(()) }
        (Left(Some(opt)), name) => opt.validate(name, rargs, residual_args),
        (Right(Some(cmd)), name) => cmd.validate(name, rargs, residual_args),
      } {
        Err(msg) => if residual_args.len() != 0 {
          return Err(format!("Unexpected argument : {:s}.", residual_args.shift()));
        } else {
          return Err(msg);
        },
        Ok(_) => {}
      }
    }
    Ok(())
  }

  fn add_option(&mut self, long_name: Option<&'static str>,
                short_name: Option<char>, description: Option<&'static str>,
                flags: uint) -> Result<Opt, &'static str> {

    let opt = Opt::new(long_name, short_name, description, flags,
                       Rc::from_mut(RefCell::new(Res { passed:0, values: ~[] })));
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

    if !opt.has_flag(Flags::Hidden) {
      self.print_options.push(opt.clone());
    }
    Ok(opt)
  }
}

pub trait OptGroup {
  /// Specify valid options for your program. Return Err() if
  /// the option has neither short nor long name or if an option
  /// with the same name was already added.
  fn add_option(&mut self, lname: Option<&'static str>,
                sname: Option<char>, description: Option<&'static str>,
                flags: uint) -> Result<Opt, &'static str>;
  /// Helper function to add a long option with Flags::Default.
  /// Fails if an option with the same name already exists.
  fn add_lopt(&mut self, name: &'static str, description: &'static str) -> Opt;
  /// Helper function to add a short option with Flags::Default.
  /// Fails if an option with the same name already exists.
  fn add_sopt(&mut self, name: char, description: &'static str) -> Opt;
  /// Helper function to add an option, which has both a long and a short name,
  /// with Flags::Default.
  /// Fails if an option with the same names already exists.
  fn add_opt(&mut self, lname: &'static str, sname: char,
             description: &'static str) -> Opt;
}

pub struct Context {
  // The arguments provided by the user.
  priv raw_args: ~[RawArg],
  // The arguments left after validation
  priv residual_args: ~[~str],
  // The context containing all the global options.
  priv inner_ctx: LocalContext,
  // The map of the authorized commands.
  priv commands: HashMap<&'static str, Cmd>,
}

impl Context {
  pub fn new(description: &'static str, args: ~[~str]) -> Context {
    Context {
      raw_args: Context::prep_args(args),
      residual_args: ~[],
      inner_ctx: LocalContext::new(description),
      commands: HashMap::new(),
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

  /// Specify valid commands for your program. Use the 'op' parameters to add 
  /// the options for this command. Fail if a command with the same name
  /// was already added.
  pub fn add_cmd_with<T>(&mut self, name: &'static str,
                         description: &'static str,
                         op: |cmd: &mut Cmd| -> T) -> (CmdRes, T) {
    let (res, cmd) = self.add_command(name, description).unwrap();
    (res, op(cmd))
  }

  /// Specify valid commands for your program. Return Err() if
  /// an option with the same name was already added.
  pub fn add_command<'a>(&'a mut self, name: &'static str,
                     description: &'static str)
                     -> Result<(CmdRes, &'a mut Cmd), &'static str> {

    if !self.commands.insert(name, Cmd::new(description)) {
      return Err("This command was already added");
    }

    // Is there a better way to get a mut ref to the value we've just
    // inserted, without doing a lookup ?
    let cmd = self.commands.get_mut(&name);
    Ok((cmd.result.clone(), cmd))
  }

  /// Validate the input arguments against the options specified via add_option().
  /// Return an Err() when the input isn't valid.
  pub fn validate(&mut self) -> Result<(), ~str> {
    self.inner_ctx.parse(&mut self.commands, &mut self.raw_args,
                         &mut self.residual_args)
  }

  /// Get an array containing the residual arguments.
  pub fn get_args<'a>(&'a self) -> &'a[~str] {
    self.residual_args.as_slice()
  }

  pub fn print_help(&self, msg: Option<&str>) {
    match msg {
      Some(err) => println!("Error : {:s}", err), None => {}
    }
    print("Usage: \n  ");
    println(self.inner_ctx.description);
    println("Valid options :");
    for opt in self.inner_ctx.print_options.iter() {
      self.print_opt(opt);
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

// Hopefully all the deduplication in here can be avoided once trait inheritance works
impl OptGroup for Cmd {
  fn add_option(&mut self, lname: Option<&'static str>,
                sname: Option<char>, description: Option<&'static str>,
                flags: uint) -> Result<Opt, &'static str> {
    self.inner_ctx.add_option(lname, sname, description, flags)
  }

  fn add_lopt(&mut self, name: &'static str, description: &'static str) -> Opt {
    self.inner_ctx.add_option(Some(name), None, Some(description),
                              Flags::Defaults).unwrap()
  }

  fn add_sopt(&mut self, name: char, description: &'static str) -> Opt {
    self.inner_ctx.add_option(None, Some(name), Some(description),
                              Flags::Defaults).unwrap()
  }

  fn add_opt(&mut self, lname: &'static str, sname: char,
             description: &'static str) -> Opt {
    self.inner_ctx.add_option(Some(lname), Some(sname), Some(description),
                              Flags::Defaults).unwrap()
  }
}

impl OptGroup for Context {
  fn add_option(&mut self, lname: Option<&'static str>,
                sname: Option<char>, description: Option<&'static str>,
                flags: uint) -> Result<Opt, &'static str> {
    self.inner_ctx.add_option(lname, sname, description, flags)
  }

  fn add_lopt(&mut self, name: &'static str, description: &'static str) -> Opt {
    self.inner_ctx.add_option(Some(name), None, Some(description),
                              Flags::Defaults).unwrap()
  }

  fn add_sopt(&mut self, name: char, description: &'static str) -> Opt {
    self.inner_ctx.add_option(None, Some(name), Some(description),
                              Flags::Defaults).unwrap()
  }

  fn add_opt(&mut self, lname: &'static str, sname: char,
             description: &'static str) -> Opt {
    self.inner_ctx.add_option(Some(lname), Some(sname), Some(description),
                              Flags::Defaults).unwrap()
  }
}

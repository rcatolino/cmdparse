#[desc = "Library to parse simple command line arguments"];
#[license = "MIT"];

// argument : string passed by the user via the command line
// command : kind of argument that is unique, doesn't start with '-',
// option : kind of argument that starts with '-' or '--', has an optional value.
// value : kind argument that is anonymous and has a value.
//         Can only be last or followed by other values.
use std::hashmap::HashMap;
use std::result::Result;

pub mod Flags {
  pub static Defaults: uint = 0;
  pub static AcceptMultiple: uint = 1 << 0;
  pub static Hidden: uint = 1 << 1;
  pub static TakesArg: uint = 1 << 2;
  pub static TakesOptionalArg: uint = 1 << 3;
}

pub struct Opt {
  long_name: Option<&'static str>,
  short_name: Option<&'static str>,
  description: Option<&'static str>,
  value: Option<~FromStr>,
  flags: uint
}

pub struct Cmd {
  name: &'static str,
  description: Option<&'static str>,
  options: ~[~Opt]
}

pub struct OptContext {
  // A summary describing the application and/or an exemple.
  summary: &'static str,
  // A list of globally valid options.
  options: ~[~Opt],
  // A list of valid commands.
  commands: ~[~Cmd],
  // The arguments provided by the user.
  raw_args: ~HashMap<~str, ~RawArg>
}

priv struct RawArg {
  checked: bool,      // Once a call to check_option has matched a RawArg.
  option: bool,       // Options start with - or --
  position: ~[uint],      // Postion in the user provided arg list. Starts at 0.
                      // The program name is ignored.
}

impl Opt {
  fn new(long: Option<&'static str>,
         short: Option<&'static str>,
         descr: Option<&'static str>,
         flags: uint) -> Result<~Opt, &'static str> {
    if long.is_none() && short.is_none() {
      Err("An option needs either a short or a long name")
    } else {
      Ok(~Opt {
        long_name: long, short_name: short, description: descr,
        value: None, flags: flags
      })
    }
  }

  fn has_flags(&self, flags: uint) -> bool {
    self.flags & flags == flags
  }
}

impl RawArg {
  pub fn new(pos: uint) -> RawArg {
    RawArg {
      checked: false, option: false, position: ~[pos]
    }
  }
}

impl OptContext {

  fn parse(args: ~[~str]) -> ~HashMap<~str, ~RawArg> {
    let mut map = ~HashMap::new();
    let mut i = 0;

    for arg in args.move_iter().skip(1) {
      // Create a new raw arg
      let mut rarg = ~RawArg::new(i);
      i += 1;
      // Check if this first character is '-'
      let key: ~str =
        if (arg[0] == '-' as u8) {
          // this is an option
          rarg.option = true;
          if (arg[1] == '-' as u8) {
            arg.slice_from(2).to_owned()
          } else {
            arg.slice_from(1).to_owned()
          }
        } else {
          arg
        };

      map.insert_or_update_with(key, rarg, |_, varg| {
        varg.position.push(i);
      });
    }

    map
  }

  pub fn new(description: &'static str, args: ~[~str]) -> OptContext {
    OptContext {
      summary: description,
      options: ~[],   // Valid options
      commands: ~[],  // Valid commands
      raw_args: OptContext::parse(args),
    }
  }

  pub fn create_option<'a>(&'a mut self,
                           long: Option<&'static str>,
                           short: Option<&'static str>,
                           descr: Option<&'static str>,
                           flags: uint) ->
    Result<&'a mut OptContext, &'static str> {

    match Opt::new(long, short, descr, flags) {
      Ok(opt) => Ok(self.add_option(opt)),
      Err(msg) => Err(msg),
    }
  }

  pub fn add_option<'a>(&'a mut self, opt: ~Opt) -> &'a mut OptContext {
    self.options.push(opt);
    self
  }

  pub fn print_help(&self) {
    print("Usage: \n  ");
    println(self.summary);
    println("Valid options :");
    for opt in self.options.iter() {
      print("  ");
      match opt.short_name {
        Some(value) => print!("-{:s},\t", value),
        None => print("\t")
      }
      match opt.long_name {
        Some(value) => print!("--{:s},\t\t", value),
        None => print("\t\t\t")
      }
      match opt.description {
        Some(value) => println(value),
        None => println("")
      }
    }
  }

}

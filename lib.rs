#[desc = "Library to parse simple command line arguments"]
#[license = "MIT"]

// argument : string passed by the user via the command line
// command : kind of argument that is unique, doesn't start with '-',
// option : kind of argument that starts with '-' or '--', has an optional value.
// value : kind argument that is anonymous and has a value.
//         Can only be last or followed by other values.
use std::hashmap::HashMap;
use std::result::Result;

pub enum OptValue {
  StrValue(&'static str),
  IntValue(int),
  BoolValue(bool),
  Present,
  Empty
}

pub enum OptValueType {
  Str,
  Int,
  Bool,
  NoValue
}

pub struct Opt {
  long_name: Option<&'static str>,
  short_name: Option<&'static str>,
  description: Option<&'static str>,
  value_type: OptValueType,
  value: OptValue
}

pub struct Cmd {
  name: &'static str,
  description: Option<&'static str>,
  options: ~[~Opt],
  value: OptValue,
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

impl Opt {
  fn is_named(&self) -> bool {
    if self.long_name.is_none() && self.short_name.is_none() {
      return false;
    }
    return true;
  }

  fn is_valid(&self) -> Result<(), &'static str> {
    if !self.is_named() {
      match self.value_type {
        NoValue => return Err("Anonymous arguments always have a value"),
        _ => {}
      }
    }

    match self.value {
      // Check if the default value for the option matches the
      // option value type.
      StrValue(_) => match self.value_type {
        Str => {}, _ => return Err("Unexpected string as default value in option.")
      },
      IntValue(_) => match self.value_type {
        Int => {}, _ => return Err("Unexpected int as default value in option.")
      },
      BoolValue(_) => match self.value_type {
        Bool => {}, _ => return Err("Unexpected boolean as default value in option.")
      },
      Present => match self.value_type {
        _ => return Err("Unexpected 'Present' as default value in option.")
      },
      Empty => {}
    }

    return Ok(());
  }
}

priv struct RawArg {
  checked: bool,      // Once a call to check_option has matched a RawArg.
  option: bool,       // Options start with - or --
  position: ~[uint],      // Postion in the user provided arg list. Starts at 0.
                      // The program name is ignored.
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
                           val_type: OptValueType,
                           default_val: OptValue) ->
    Result<&'a mut OptContext, &'static str> {

    self.add_option(~Opt { long_name: long,
                           short_name: short,
                           description: descr,
                           value_type: val_type,
                           value: default_val })
  }

  pub fn add_option<'a>(&'a mut self, opt: ~Opt) ->
    Result<&'a mut OptContext, &'static str> {
    match opt.is_valid() {
      Err(msg) => return Err(msg),
      _ => {}
    }

    self.options.push(opt);
    Ok(self)
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

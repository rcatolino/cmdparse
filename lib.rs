#[desc = "Library to parse simple command line arguments"]
#[license = "MIT"]

use std::result::Result;
use std::hashmap::HashMap;

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

pub struct OptContext<'self> {
  summary: &'static str,
  options: ~[~Opt],
  arguments: ~[~Opt],
  values: Option<~HashMap<&'static str, &'self ~Opt>>
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

impl<'self> OptContext<'self> {

  pub fn new(description: &'static str) -> ~OptContext {
    ~OptContext {
      summary: description,
      options: ~[],
      arguments: ~[],
      values: None
    }
  }

  pub fn create_option<'a>(&'a mut self,
                           long: Option<&'static str>,
                           short: Option<&'static str>,
                           descr: Option<&'static str>,
                           val_type: OptValueType,
                           default_val: OptValue) ->
    Result<&'a mut OptContext<'self>, &'static str> {

    self.add_option(~Opt { long_name: long,
                           short_name: short,
                           description: descr,
                           value_type: val_type,
                           value: default_val })
  }

  pub fn add_option<'a>(&'a mut self, opt: ~Opt) ->
    Result<&'a mut OptContext<'self>, &'static str> {
    match opt.is_valid() {
      Err(msg) => return Err(msg),
      _ => {}
    }

    if !opt.is_named() {
      self.arguments.push(opt);
    } else {
      self.options.push(opt);
    }

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

  pub fn get_option_value<'a>(&'a self) -> Option<&'a OptValue> {
    None
  }

  pub fn prepare(&'self mut self) ->
    Result<(), &'static str> {
    if self.values.is_some() {
      return Err("Cannot call parse_args more than once");
    }

    // Organize the valid options into a map
    let mut map: ~HashMap<&'static str, &'self ~Opt> = ~HashMap::new();
    for opt in self.options.iter() {
      match opt.short_name {
        Some(name) => if map.swap(name, opt).is_some() {
          return Err("Option containing this name already inserted.");
        },
        None => {}
      }

      match opt.long_name {
        Some(name) => if map.swap(name, opt).is_some() {
          return Err("Option containing this name already inserted.");
        },
        None => {}
      }
    }

    self.values = Some(map);
    Ok(())
  }

  pub fn parse_args(&mut self, args: ~[~str]) -> Result<(),&'static str> {
    // Go over all the program args to see if they match the options
    // Skip the program name
    for arg in args.iter().skip(1) {
      println(*arg);
    }

    Ok(())
  }
}


extern mod cmdparse;

use cmdparse::OptContext;
use cmdparse::Flags;

#[test]
fn test_add_option_valid() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options with no value:
  ctx.add_option(Some("long"), Some("a"), Some("description"), Flags::Defaults).unwrap();
  ctx.add_option(Some("long2"), None, Some("description"), Flags::Defaults).unwrap();
  ctx.add_option(Some("long3"), Some("b"), None, Flags::Defaults).unwrap();
  ctx.add_option(Some("long4"), None, None, Flags::Defaults).unwrap();
  ctx.add_option(None, Some("c"), Some("description"), Flags::Defaults).unwrap();
  ctx.add_option(None, Some("d"), None, Flags::Defaults).unwrap();
}

#[test]
fn test_check_result() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  let d_opt = ctx.add_option(None, Some("d"), None, Flags::Defaults).unwrap();
  let e_opt = ctx.add_option(None, Some("e"), None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| ctx.print_help(Some(msg)));
  ctx.check(d_opt);
  let d_val: int = match ctx.take_value(e_opt) {
    Left(value) => value,
    Right(_) => 0
  };
  println(d_val.to_str());
}

#[test]
fn test_add_option_invalid_same_name() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options with no value:
  ctx.add_option(Some("long"), Some("a"), None, Flags::Defaults).unwrap();
  ctx.add_option(Some("long"), None, None, Flags::Defaults).unwrap_err();
  ctx.add_option(Some("long2"), Some("a"), None, Flags::Defaults).unwrap_err();
}

#[test]
fn test_add_option_invalid() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options which take values but have no Flags::Defaults
  ctx.add_option(None, None, None, Flags::Defaults).unwrap_err();
  ctx.add_option(None, None, Some("description"), Flags::Defaults).unwrap_err();
}

#[test]
fn test_add_option_print() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  ctx.add_option(Some("long4"), Some("s"), Some("Description"), Flags::Defaults).unwrap();
  ctx.add_option(None, Some("t"), Some("Description"), Flags::Defaults).unwrap();
  ctx.add_option(None, Some("u"), Some("Description"), Flags::TakesOptionalArg).unwrap();
  ctx.add_option(None, Some("v"), Some("Description"), Flags::TakesArg).unwrap();
  ctx.add_option(Some("long5"), Some("w"), Some("Description"), Flags::TakesArg).unwrap();
  ctx.add_option(Some("long6"), None, Some("Description"), Flags::TakesOptionalArg).unwrap();
  ctx.add_option(Some("long7"), None, None, Flags::Defaults).unwrap();
  ctx.print_help(None);
}


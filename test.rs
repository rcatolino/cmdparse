extern mod cmdparse;

use cmdparse::OptContext;
use cmdparse::Flags;

// Tests for the options creation
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

// Tests for the validation.
#[test]
fn test_check_validation_invalid1() {
  let args = ~[~"test", ~"-i"];
  let mut ctx = OptContext::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(None, Some("d"), None, Flags::Defaults).unwrap();
  match ctx.validate() {
    Err(msg) => ctx.print_help(Some(msg.as_slice())),
    Ok(()) => assert!(false),
  }
  assert!(ctx.check(d_opt) == false);
}

#[test]
fn test_check_validation_invalid2() {
  let args = ~[~"test", ~"--long1"];
  let mut ctx = OptContext::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(None, Some("d"), None, Flags::Defaults).unwrap();
  match ctx.validate() {
    Err(msg) => ctx.print_help(Some(msg.as_slice())),
    Ok(()) => assert!(false),
  }
  assert!(ctx.check(d_opt) == false);
}

#[test]
fn test_check_validation_invalid3() {
  let args = ~[~"test", ~"--long1", ~"invalidarg"];
  let mut ctx = OptContext::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(Some("long1"), Some("d"), None, Flags::Defaults).unwrap();
  match ctx.validate() {
    Err(msg) => ctx.print_help(Some(msg.as_slice())),
    Ok(()) => assert!(false),
  }
  ctx.check(d_opt);
}

#[test]
fn test_check_validation_invalid4() {
  let args = ~[~"test", ~"invalidarg", ~"--long1"];
  let mut ctx = OptContext::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(Some("long1"), Some("d"), None, Flags::Defaults).unwrap();
  match ctx.validate() {
    Err(msg) => ctx.print_help(Some(msg.as_slice())),
    Ok(()) => assert!(false),
  }
  ctx.check(d_opt);
}

// Tests for the actual results.
#[test]
fn test_check_result_no_value_no_flags_valid() {
  let args = ~[~"test", ~"-d", ~"--long1"];
  let mut ctx = OptContext::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(None, Some("d"), None, Flags::Defaults).unwrap();
  let e_opt = ctx.add_option(None, Some("e"), None, Flags::Defaults).unwrap();
  let g_opt = ctx.add_option(Some("long1"), Some("g"), None, Flags::Defaults).unwrap();
  let h_opt = ctx.add_option(Some("long2"), Some("h"), None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  assert!(ctx.check(d_opt) == true);
  assert!(ctx.check(e_opt) == false);
  assert!(ctx.check(g_opt) == true);
  assert!(ctx.check(h_opt) == false);
}

#[test]
fn test_check_result_single_value_no_flags() {
  let args = ~[~"test", ~"-f", ~"value"];
  let mut ctx = OptContext::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some("e"), None, Flags::Defaults).unwrap();
  //ctx.validate().map_err(|msg| ctx.print_help(Some(msg.as_slice())));
  let e_val: int = match ctx.take_value(e_opt) {
    Left(value) => value,
    Right(_) => 0
  };
  println(e_val.to_str());
}

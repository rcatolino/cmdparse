extern mod cmdparse;

use cmdparse::Context;
use cmdparse::Flags;

// Tests for the options creation
#[test]
fn test_add_option_valid() {
  let mut ctx = Context::new("test [option] [argument]", ~[~"test"]);
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
  let mut ctx = Context::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options with no value:
  ctx.add_option(Some("long"), Some("a"), None, Flags::Defaults).unwrap();
  ctx.add_option(Some("long"), None, None, Flags::Defaults).unwrap_err();
  ctx.add_option(Some("long2"), Some("a"), None, Flags::Defaults).unwrap_err();
}

#[test]
fn test_add_option_invalid() {
  let mut ctx = Context::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options which take values but have no Flags::Defaults
  ctx.add_option(None, None, None, Flags::Defaults).unwrap_err();
  ctx.add_option(None, None, Some("description"), Flags::Defaults).unwrap_err();
}

// Tests for the validation.
#[test]
fn test_check_validation_invalid1() {
  let args = ~[~"test", ~"-i"];
  let mut ctx = Context::new("test [option] [argument]", args);
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
  let mut ctx = Context::new("test [option] [argument]", args);
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
  let mut ctx = Context::new("test [option] [argument]", args);
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
  let mut ctx = Context::new("test [option] [argument]", args);
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
  let mut ctx = Context::new("test [option] [argument]", args);
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
fn test_check_result_single_value_valid_int() {
  let args = ~[~"test", ~"-i", ~"33"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some("i"), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match ctx.take_value(e_opt) {
    Left(Some(value)) => value,
    Left(None) => {assert!(false); 0}
    Right(_) => {assert!(false); 0}
  };
  assert!(e_val == 33);
}

#[test]
fn test_check_result_single_value_valid_bool() {
  let args = ~[~"test", ~"-b", ~"true"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some("b"), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match ctx.take_value(e_opt) {
    Left(Some(value)) => value,
    Left(None) => {assert!(false); false}
    Right(_) => {assert!(false); false}
  };
  assert!(e_val == true);
}

#[test]
fn test_check_result_single_value_valid_str() {
  let args = ~[~"test", ~"-s", ~"value"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some("s"), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match ctx.take_value(e_opt) {
    Left(Some(value)) => value,
    Left(None) => {assert!(false); ~"eror"}
    Right(_) => {assert!(false); ~"error"}
  };
  assert!(e_val == ~"value");
}

#[test]
fn test_check_result_single_value_valid_float() {
  let args = ~[~"test", ~"-f", ~"1.5"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some("f"), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match ctx.take_value(e_opt) {
    Left(Some(value)) => value,
    Left(None) => {assert!(false); 0f32}
    Right(_) => {assert!(false); 0f32}
  };
  assert!(e_val == 1.5);
}

#[test]
fn test_check_result_single_value_invalid_int() {
  let args = ~[~"test", ~"-i", ~"invalid"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some("i"), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match ctx.take_value(e_opt) {
    Left(Some(value)) => {assert!(false); value}
    Left(None) => {0}
    Right(_) => {assert!(false); 0}
  };
  assert!(e_val == 0);
}

#[test]
fn test_check_result_single_value_invalid_bool() {
  let args = ~[~"test", ~"-b", ~"invalid"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some("b"), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match ctx.take_value(e_opt) {
    Left(Some(value)) => {assert!(false); value}
    Left(None) => {false}
    Right(_) => {assert!(false); false}
  };
  assert!(e_val == false);
}

#[test]
fn test_check_result_single_value_invalid_float() {
  let args = ~[~"test", ~"-f", ~"invalid"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some("f"), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match ctx.take_value(e_opt) {
    Left(Some(value)) => {assert!(false); value}
    Left(None) => {0f32}
    Right(_) => {assert!(false); 0f32}
  };
  assert!(e_val == 0f32);
}

#[test]
fn test_check_result_single_value_unpassed1() {
  let args = ~[~"test", ~"-s", ~"-a"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some("s"), None, Flags::TakesOptionalArg).unwrap();
  let a_opt = ctx.add_option(None, Some("a"), None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match ctx.take_value(e_opt) {
    Left(Some(value)) => {assert!(false); value}
    Left(None) => {assert!(false); 0}
    Right(passed) => {assert!(passed); 0}
  };
  assert!(e_val == 0);
  ctx.check(a_opt);
}

#[test]
fn test_check_result_single_value_unpassed2() {
  let args = ~[~"test", ~"-a"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some("s"), None, Flags::TakesOptionalArg).unwrap();
  let a_opt = ctx.add_option(None, Some("a"), None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match ctx.take_value(e_opt) {
    Left(Some(value)) => {assert!(false); value}
    Left(None) => {assert!(false); 0}
    Right(passed) => {assert!(!passed); 0}
  };
  assert!(e_val == 0);
  ctx.check(a_opt);
}


// Tests for the 'Unique' flag
#[test]
fn test_check_result_no_value_no_flags_multiple() {
  let args = ~[~"test", ~"-d", ~"--long1", ~"-d", ~"--long1"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(None, Some("d"), None, Flags::Defaults).unwrap();
  let e_opt = ctx.add_option(None, Some("e"), None, Flags::Defaults).unwrap();
  let g_opt = ctx.add_option(Some("long1"), Some("g"), None, Flags::Defaults).unwrap();
  let h_opt = ctx.add_option(Some("long2"), Some("h"), None, Flags::Defaults).unwrap();
  let l3_opt = ctx.add_option(Some("long3"), None, None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  assert!(ctx.count(d_opt) == 2);
  assert!(ctx.check(e_opt) == false);
  assert!(ctx.count(g_opt) == 2);
  assert!(ctx.check(h_opt) == false);
  assert!(ctx.count(l3_opt) == 0);
}

#[test]
fn test_check_result_no_value_unique() {
  let args = ~[~"test", ~"-d", ~"--long3", ~"-d"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(None, Some("d"), None, Flags::Unique).unwrap();
  let l3_opt = ctx.add_option(Some("long3"), None, None, Flags::Defaults).unwrap();
  match ctx.validate() {
    Err(msg) => ctx.print_help(Some(msg.as_slice())),
    Ok(()) => assert!(false)
  }
  ctx.count(d_opt);
  ctx.count(l3_opt);
}

#[test]
fn test_check_result_no_value_unique2() {
  let args = ~[~"test", ~"-d", ~"--long3", ~"-d"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(None, Some("d"), None, Flags::Unique).unwrap();
  let l3_opt = ctx.add_option(Some("long3"), None, None, Flags::TakesOptionalArg).unwrap();
  match ctx.validate() {
    Err(msg) => ctx.print_help(Some(msg.as_slice())),
    Ok(()) => assert!(false)
  }
  ctx.count(d_opt);
  ctx.count(l3_opt);
}

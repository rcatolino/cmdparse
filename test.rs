extern mod cmdparse;

use cmdparse::{Context,OptGroup,Flags};
use std::str;

// Tests for the options creation
#[test]
fn test_add_option_valid() {
  let mut ctx = Context::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options with no value:
  ctx.add_option(Some("long"), Some('a'), Some("description"), Flags::Defaults).unwrap();
  ctx.add_option(Some("long2"), None, Some("description"), Flags::Defaults).unwrap();
  ctx.add_option(Some("long3"), Some('b'), None, Flags::Defaults).unwrap();
  ctx.add_option(Some("long4"), None, None, Flags::Defaults).unwrap();
  ctx.add_option(None, Some('c'), Some("description"), Flags::Defaults).unwrap();
  ctx.add_option(None, Some('d'), None, Flags::Defaults).unwrap();
}

#[test]
fn test_add_option_invalid_same_name() {
  let mut ctx = Context::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options with no value:
  ctx.add_option(Some("long"), Some('a'), None, Flags::Defaults).unwrap();
  ctx.add_option(Some("long"), None, None, Flags::Defaults).unwrap_err();
  ctx.add_option(Some("long2"), Some('a'), None, Flags::Defaults).unwrap_err();
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
  let d_opt = ctx.add_option(None, Some('d'), None, Flags::Defaults).unwrap();
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
  let d_opt = ctx.add_option(None, Some('d'), None, Flags::Defaults).unwrap();
  match ctx.validate() {
    Err(msg) => ctx.print_help(Some(msg.as_slice())),
    Ok(()) => assert!(false),
  }
  assert!(ctx.check(d_opt) == false);
}

#[test]
fn test_check_validation_invalid4() {
  let args = ~[~"test", ~"invalidarg", ~"--long1"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(Some("long1"), Some('d'), None, Flags::Defaults).unwrap();
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
  let d_opt = ctx.add_option(None, Some('d'), None, Flags::Defaults).unwrap();
  let e_opt = ctx.add_option(None, Some('e'), None, Flags::Defaults).unwrap();
  let g_opt = ctx.add_option(Some("long1"), Some('g'), None, Flags::Defaults).unwrap();
  let h_opt = ctx.add_option(Some("long2"), Some('h'), None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  assert!(ctx.check(d_opt) == true);
  assert!(ctx.check(e_opt) == false);
  assert!(ctx.check(g_opt) == true);
  assert!(ctx.check(h_opt) == false);
}

#[test]
fn test_check_result_no_value_no_flags_grouped() {
  let args = ~[~"test", ~"-deg", ~"--long1"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(None, Some('d'), None, Flags::Defaults).unwrap();
  let e_opt = ctx.add_option(None, Some('e'), None, Flags::Defaults).unwrap();
  let f_opt = ctx.add_option(None, Some('f'), None, Flags::Defaults).unwrap();
  let g_opt = ctx.add_option(None, Some('g'), None, Flags::Defaults).unwrap();
  let h_opt = ctx.add_option(Some("long1"), Some('h'), None, Flags::Defaults).unwrap();
  let i_opt = ctx.add_option(Some("long2"), Some('i'), None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  assert!(ctx.check(d_opt) == true);
  assert!(ctx.check(e_opt) == true);
  assert!(ctx.check(f_opt) == false);
  assert!(ctx.check(g_opt) == true);
  assert!(ctx.check(h_opt) == true);
  assert!(ctx.check(i_opt) == false);
}

#[test]
fn test_check_result_grouped_value_valid() {
  let args = ~[~"test", ~"-deg", ~"value", ~"--long1"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(None, Some('d'), None, Flags::Defaults).unwrap();
  let e_opt = ctx.add_option(None, Some('e'), None, Flags::Defaults).unwrap();
  let f_opt = ctx.add_option(None, Some('f'), None, Flags::Defaults).unwrap();
  let g_opt = ctx.add_option(None, Some('g'), None, Flags::TakesArg).unwrap();
  let h_opt = ctx.add_option(Some("long1"), Some('h'), None, Flags::Defaults).unwrap();
  let i_opt = ctx.add_option(Some("long2"), Some('i'), None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  assert!(ctx.check(d_opt) == true);
  assert!(ctx.check(e_opt) == true);
  assert!(ctx.check(f_opt) == false);
  assert!(ctx.check(h_opt) == true);
  assert!(ctx.check(i_opt) == false);
  match ctx.take_value::<~str>(g_opt) {
    Left(Some(val)) => assert!(val == ~"value"),
    Left(None) => assert!(false),
    Right(_) => assert!(false),
  }
}

#[test]
fn test_check_result_grouped_value_invalid() {
  let args = ~[~"test", ~"-deg", ~"--long1"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(None, Some('d'), None, Flags::Defaults).unwrap();
  let e_opt = ctx.add_option(None, Some('e'), None, Flags::TakesOptionalArg).unwrap();
  let f_opt = ctx.add_option(None, Some('f'), None, Flags::Defaults).unwrap();
  let g_opt = ctx.add_option(None, Some('g'), None, Flags::Defaults).unwrap();
  let h_opt = ctx.add_option(Some("long1"), Some('h'), None, Flags::Defaults).unwrap();
  let i_opt = ctx.add_option(Some("long2"), Some('i'), None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  assert!(ctx.check(d_opt) == true);
  assert!(ctx.check(g_opt) == true);
  assert!(ctx.check(f_opt) == false);
  assert!(ctx.check(h_opt) == true);
  assert!(ctx.check(i_opt) == false);
  match ctx.take_value::<~str>(e_opt) {
    Left(_) => assert!(false),
    Right(val) => assert!(val),
  }
}

#[test]
fn test_check_result_long_option_values() {
  let args = ~[~"test", ~"--long1=value1", ~"-d", ~"--long2", ~"value2"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(None, Some('d'), None, Flags::Defaults).unwrap();
  let g_opt = ctx.add_option(Some("long1"), Some('g'), None, Flags::TakesArg).unwrap();
  let e_opt = ctx.add_option(None, Some('e'), None, Flags::Defaults).unwrap();
  let h_opt = ctx.add_option(Some("long2"), Some('h'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  assert!(ctx.check(d_opt) == true);
  assert!(ctx.check(e_opt) == false);
  match ctx.take_value::<~str>(g_opt) {
    Left(Some(value)) => assert!(value == ~"value1"),
    _ => assert!(false)
  }
  match ctx.take_value::<~str>(h_opt) {
    Left(Some(value)) => assert!(value == ~"value2"),
    _ => assert!(false)
  }
}

#[test]
fn test_check_result_single_value_valid_int() {
  let args = ~[~"test", ~"-i", ~"33"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('i'), None, Flags::TakesArg).unwrap();
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
  let e_opt = ctx.add_option(None, Some('b'), None, Flags::TakesArg).unwrap();
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
  let e_opt = ctx.add_option(None, Some('s'), None, Flags::TakesArg).unwrap();
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
  let e_opt = ctx.add_option(None, Some('f'), None, Flags::TakesArg).unwrap();
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
  let e_opt = ctx.add_option(None, Some('i'), None, Flags::TakesArg).unwrap();
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
  let e_opt = ctx.add_option(None, Some('b'), None, Flags::TakesArg).unwrap();
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
  let e_opt = ctx.add_option(None, Some('f'), None, Flags::TakesArg).unwrap();
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
  let e_opt = ctx.add_option(None, Some('s'), None, Flags::TakesOptionalArg).unwrap();
  let a_opt = ctx.add_option(None, Some('a'), None, Flags::Defaults).unwrap();
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
  let e_opt = ctx.add_option(None, Some('s'), None, Flags::TakesOptionalArg).unwrap();
  let a_opt = ctx.add_option(None, Some('a'), None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match ctx.take_value(e_opt) {
    Left(Some(value)) => {assert!(false); value}
    Left(None) => {assert!(false); 0}
    Right(passed) => {assert!(!passed); 0}
  };
  assert!(e_val == 0);
  ctx.check(a_opt);
}

#[test]
fn test_check_result_multiple_values_int() {
  let args = ~[~"test", ~"-i", ~"33" , ~"-i", ~"32", ~"--int=31", ~"--int", ~"30"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(Some("int"), Some('i'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  match ctx.take_values::<int>(e_opt) {
    Left(values) => for (val, expected) in values.move_iter().filter_map(|opt_val| {
      opt_val.or_else(|| { assert!(false); None})
    }).zip((~[33, 32, 31, 30]).move_iter()) {
      assert!(val == expected);
    },
    Right(_) => assert!(false)
  }
}

#[test]
fn test_check_result_multiple_values_int_invalid() {
  let args = ~[~"test", ~"-i", ~"33" , ~"-i", ~"notanint", ~"--int=31", ~"--int", ~"30"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(Some("int"), Some('i'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  match ctx.take_values::<int>(e_opt) {
    Left(values) => for (val, expected) in values.move_iter().
      filter_map(|opt_val| opt_val).zip((~[33, 31, 30]).move_iter()) {
      assert!(val == expected);
    },
    Right(_) => assert!(false)
  }
}

#[test]
fn test_check_result_multiple_values_some() {
  let args = ~[~"test", ~"-i", ~"33" , ~"-i", ~"--int=31", ~"--int", ~"30"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(Some("int"), Some('i'), None, Flags::TakesOptionalArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  match ctx.take_values::<int>(e_opt) {
    Left(values) => for (val, expected) in values.move_iter().filter_map(|opt_val| {
      opt_val.or_else(|| { assert!(false); None})
    }).zip((~[33, 31, 30]).move_iter()) {
      assert!(val == expected);
    },
    Right(_) => assert!(false)
  }
}

#[test]
fn test_check_result_multiple_values_none() {
  let args = ~[~"test"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(Some("int"), Some('i'), None, Flags::TakesOptionalArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  match ctx.take_values::<int>(e_opt) {
    Left(_) => assert!(false),
    Right(nb) => assert!(nb == 0)
  }
}

#[test]
fn test_check_result_multiple_values_unpassed() {
  let args = ~[~"test", ~"-i", ~"--int"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(Some("int"), Some('i'), None, Flags::TakesOptionalArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  match ctx.take_values::<int>(e_opt) {
    Left(_) => assert!(false),
    Right(nb) => assert!(nb == 2)
  }
}

// Tests for the anonymous arguments

#[test]
fn test_check_validation_valid_argument1() {
  let args = ~[~"test", ~"--long1", ~"validarg1", ~"validarg2"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(Some("long1"), Some('d'), None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  for (arg, expected) in ctx.get_args().iter().
    zip((~["validarg1", "validarg2"]).move_iter()) {
    assert!(str::eq_slice(*arg, expected));
  }
  assert!(ctx.check(d_opt));
}

#[test]
fn test_check_validation_valid_argument2() {
  let args = ~[~"test", ~"--long1", ~"validarg1", ~"validarg2"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(Some("long1"), Some('d'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  for (arg, expected) in ctx.get_args().iter().
    zip((~["validarg2"]).move_iter()) {
    assert!(str::eq_slice(*arg, expected));
  }
  match ctx.take_value::<~str>(d_opt) {
    Left(val) => match val {
      Some(val) => assert!(str::eq_slice(val, "validarg1")),
      None => assert!(false)
    },
    Right(_) => assert!(false),
  }
}

// Tests for the 'Unique' flag
#[test]
fn test_check_result_no_value_no_flags_multiple() {
  let args = ~[~"test", ~"-d", ~"--long1", ~"-d", ~"--long1"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let d_opt = ctx.add_option(None, Some('d'), None, Flags::Defaults).unwrap();
  let e_opt = ctx.add_option(None, Some('e'), None, Flags::Defaults).unwrap();
  let g_opt = ctx.add_option(Some("long1"), Some('g'), None, Flags::Defaults).unwrap();
  let h_opt = ctx.add_option(Some("long2"), Some('h'), None, Flags::Defaults).unwrap();
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
  let d_opt = ctx.add_option(None, Some('d'), None, Flags::Unique).unwrap();
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
  let d_opt = ctx.add_option(None, Some('d'), None, Flags::Unique).unwrap();
  let l3_opt = ctx.add_option(Some("long3"), None, None, Flags::TakesOptionalArg).unwrap();
  match ctx.validate() {
    Err(msg) => ctx.print_help(Some(msg.as_slice())),
    Ok(()) => assert!(false)
  }
  ctx.count(d_opt);
  ctx.count(l3_opt);
}

// Tests with commands
#[test]
fn test_add_command_valid() {
  let mut ctx = Context::new("test [option] command [argument]", ~[~"test"]);
  // Those are valid options:
  let mut cmd = ctx.add_command("command", "description").unwrap();
}

#[test]
fn test_add_command_invalid_same_name() {
  let mut ctx = Context::new("test [option] command [argument]", ~[~"test"]);
  {
  let cmd1 = ctx.add_command("command", "description").unwrap();
  }
  {
  let cmd2 = ctx.add_command("command", "description2").unwrap_err();
  }
}

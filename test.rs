#![allow(unused_must_use)]
#![cfg(test)]

extern crate cmdparse;
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
  assert!(d_opt.check() == false);
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
  assert!(d_opt.check() == false);
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
  d_opt.check();
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
  assert!(d_opt.check() == true);
  assert!(e_opt.check() == false);
  assert!(g_opt.check() == true);
  assert!(h_opt.check() == false);
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
  assert!(d_opt.check() == true);
  assert!(e_opt.check() == true);
  assert!(f_opt.check() == false);
  assert!(g_opt.check() == true);
  assert!(h_opt.check() == true);
  assert!(i_opt.check() == false);
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
  assert!(d_opt.check() == true);
  assert!(e_opt.check() == true);
  assert!(f_opt.check() == false);
  assert!(h_opt.check() == true);
  assert!(i_opt.check() == false);
  match g_opt.take_value::<~str>() {
    Ok(Some(val)) => assert!(val == ~"value"),
    Ok(None) => assert!(false),
    Err(_) => assert!(false),
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
  assert!(d_opt.check() == true);
  assert!(g_opt.check() == true);
  assert!(f_opt.check() == false);
  assert!(h_opt.check() == true);
  assert!(i_opt.check() == false);
  match e_opt.take_value::<~str>() {
    Ok(_) => assert!(false),
    Err(val) => assert!(val),
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
  assert!(d_opt.check() == true);
  assert!(e_opt.check() == false);
  match g_opt.take_value::<~str>() {
    Ok(Some(value)) => assert!(value == ~"value1"),
    _ => assert!(false)
  }
  match h_opt.take_value::<~str>() {
    Ok(Some(value)) => assert!(value == ~"value2"),
    _ => assert!(false)
  }
}

#[test]
fn test_check_result_single_value_valid_int() {
  let args = ~[~"test", ~"-i", ~"33"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('i'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match e_opt.take_value() {
    Ok(Some(value)) => value,
    Ok(None) => {assert!(false); 0}
    Err(_) => {assert!(false); 0}
  };
  assert!(e_val == 33);
}

#[test]
fn test_check_result_single_value_valid_bool() {
  let args = ~[~"test", ~"-b", ~"true"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('b'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match e_opt.take_value() {
    Ok(Some(value)) => value,
    Ok(None) => {assert!(false); false}
    Err(_) => {assert!(false); false}
  };
  assert!(e_val == true);
}

#[test]
fn test_check_result_single_value_valid_str() {
  let args = ~[~"test", ~"-s", ~"value"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('s'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match e_opt.take_value() {
    Ok(Some(value)) => value,
    Ok(None) => {assert!(false); ~"error"}
    Err(_) => {assert!(false); ~"error"}
  };
  assert!(e_val == ~"value");
}

#[test]
fn test_check_result_take_value_multiple_str() {
  let args = ~[~"test", ~"-s", ~"value", ~"-s", ~"value2"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('s'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match e_opt.take_value() {
    Ok(Some(value)) => value,
    Ok(None) => {assert!(false); ~"error"}
    Err(_) => {assert!(false); ~"error"}
  };
  let e_val2 = match e_opt.take_value() {
    Ok(Some(value)) => value,
    Ok(None) => {assert!(false); ~"error"}
    Err(_) => {assert!(false); ~"error"}
  };

  assert!(e_val == ~"value2");
  assert!(e_val2 == ~"value");
}

#[test]
fn test_check_result_single_value_valid_float() {
  let args = ~[~"test", ~"-f", ~"1.5"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('f'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match e_opt.take_value() {
    Ok(Some(value)) => value,
    Ok(None) => {assert!(false); 0f32}
    Err(_) => {assert!(false); 0f32}
  };
  assert!(e_val == 1.5);
}

#[test]
fn test_check_result_single_value_valid_bool2() {
  let args = ~[~"test", ~"-b", ~"true"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('b'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = e_opt.value_or(&ctx, false);
  assert!(e_val == true);
}

#[test]
#[should_fail]
fn test_check_result_single_value_invalid_bool2() {
  let args = ~[~"test", ~"-b", ~"value"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('b'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = e_opt.value_or(&ctx, false);
  assert!(e_val == false);
}

#[test]
fn test_check_result_single_value_invalid_bool3() {
  let args = ~[~"test", ~"-c", ~"true"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('b'), None, Flags::TakesArg).unwrap();
  ctx.add_option(None, Some('c'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = e_opt.value_or(&ctx, false);
  assert!(e_val == false);
}

#[test]
fn test_check_result_single_value_invalid_bool4() {
  let args = ~[~"test", ~"-c", ~"-d", ~"33"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let c_opt = ctx.add_option(None, Some('c'), None, Flags::TakesOptionalArg).unwrap();
  let d_opt = ctx.add_option(None, Some('d'), None, Flags::TakesOptionalArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let c_val = c_opt.value_or(&ctx, 35);
  let d_val = d_opt.value_or::<int>(&ctx, 35);
  assert!(c_val == 35);
  assert!(d_val == 33);
}

#[test]
fn test_check_result_single_value_invalid_int() {
  let args = ~[~"test", ~"-i", ~"invalid"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('i'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match e_opt.take_value() {
    Ok(Some(value)) => {assert!(false); value}
    Ok(None) => {0}
    Err(_) => {assert!(false); 0}
  };
  assert!(e_val == 0);
}

#[test]
fn test_check_result_single_value_invalid_bool() {
  let args = ~[~"test", ~"-b", ~"invalid"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('b'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match e_opt.take_value() {
    Ok(Some(value)) => {assert!(false); value}
    Ok(None) => {false}
    Err(_) => {assert!(false); false}
  };
  assert!(e_val == false);
}

#[test]
fn test_check_result_single_value_invalid_float() {
  let args = ~[~"test", ~"-f", ~"invalid"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('f'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match e_opt.take_value() {
    Ok(Some(value)) => {assert!(false); value}
    Ok(None) => {0f32}
    Err(_) => {assert!(false); 0f32}
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
  let e_val = match e_opt.take_value() {
    Ok(Some(value)) => {assert!(false); value}
    Ok(None) => {assert!(false); 0}
    Err(passed) => {assert!(passed); 0}
  };
  assert!(e_val == 0);
  a_opt.check();
}

#[test]
fn test_check_result_single_value_unpassed2() {
  let args = ~[~"test", ~"-a"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(None, Some('s'), None, Flags::TakesOptionalArg).unwrap();
  let a_opt = ctx.add_option(None, Some('a'), None, Flags::Defaults).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  let e_val = match e_opt.take_value() {
    Ok(Some(value)) => {assert!(false); value}
    Ok(None) => {assert!(false); 0}
    Err(passed) => {assert!(!passed); 0}
  };
  assert!(e_val == 0);
  a_opt.check();
}

#[test]
fn test_check_result_single_value_before_validate() {
  let args = ~[~"test", ~"-a", ~"value"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let a_opt = ctx.add_option(None, Some('a'), None, Flags::TakesArg).unwrap();
  match a_opt.take_value::<~str>() {
    Ok(Some(_)) => assert!(false),
    Ok(None) => assert!(false),
    Err(passed) => assert!(!passed),
  }
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
}

#[test]
fn test_check_result_multiple_values_int() {
  let args = ~[~"test", ~"-i", ~"33" , ~"-i", ~"32", ~"--int=31", ~"--int", ~"30"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(Some("int"), Some('i'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  match e_opt.take_values::<int>() {
    Ok(values) => for (val, expected) in values.move_iter().filter_map(|opt_val| {
      opt_val.or_else(|| { assert!(false); None})
    }).zip((~[33, 32, 31, 30]).move_iter()) {
      assert!(val == expected);
    },
    Err(_) => assert!(false)
  }
}

#[test]
fn test_check_result_multiple_values_int_invalid() {
  let args = ~[~"test", ~"-i", ~"33" , ~"-i", ~"notanint", ~"--int=31", ~"--int", ~"30"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(Some("int"), Some('i'), None, Flags::TakesArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  match e_opt.take_values::<int>() {
    Ok(values) => for (val, expected) in values.move_iter().
      filter_map(|opt_val| opt_val).zip((~[33, 31, 30]).move_iter()) {
      assert!(val == expected);
    },
    Err(_) => assert!(false)
  }
}

#[test]
fn test_check_result_multiple_values_some() {
  let args = ~[~"test", ~"-i", ~"33" , ~"-i", ~"--int=31", ~"--int", ~"30"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(Some("int"), Some('i'), None, Flags::TakesOptionalArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  match e_opt.take_values::<int>() {
    Ok(values) => for (val, expected) in values.move_iter().filter_map(|opt_val| {
      opt_val.or_else(|| { assert!(false); None})
    }).zip((~[33, 31, 30]).move_iter()) {
      assert!(val == expected);
    },
    Err(_) => assert!(false)
  }
}

#[test]
fn test_check_result_multiple_values_none() {
  let args = ~[~"test"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(Some("int"), Some('i'), None, Flags::TakesOptionalArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  match e_opt.take_values::<int>() {
    Ok(_) => assert!(false),
    Err(nb) => assert!(nb == 0)
  }
}

#[test]
fn test_check_result_multiple_values_unpassed() {
  let args = ~[~"test", ~"-i", ~"--int"];
  let mut ctx = Context::new("test [option] [argument]", args);
  let e_opt = ctx.add_option(Some("int"), Some('i'), None, Flags::TakesOptionalArg).unwrap();
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  match e_opt.take_values::<int>() {
    Ok(_) => assert!(false),
    Err(nb) => assert!(nb == 2)
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
  assert!(d_opt.check());
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
  match d_opt.take_value::<~str>() {
    Ok(val) => match val {
      Some(val) => assert!(str::eq_slice(val, "validarg1")),
      None => assert!(false)
    },
    Err(_) => assert!(false),
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
  assert!(d_opt.count() == 2);
  assert!(e_opt.check() == false);
  assert!(g_opt.count() == 2);
  assert!(h_opt.check() == false);
  assert!(l3_opt.count() == 0);
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
  d_opt.count();
  l3_opt.count();
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
  d_opt.count();
  l3_opt.count();
}

// Tests with commands
#[test]
fn test_add_command_valid() {
  let mut ctx = Context::new("test [option] command [argument]", ~[~"test"]);
  // Those are valid options:
  ctx.add_command("command", "description").unwrap();
}

#[test]
fn test_add_command_invalid_same_name() {
  let mut ctx = Context::new("test [option] command [argument]", ~[~"test"]);
  ctx.add_command("command", "description").unwrap();
  ctx.add_command("command", "description2").unwrap_err();
}

// Tests with commands and options.
#[test]
fn test_add_command_option() {
  let mut ctx = Context::new("test [option] command [command-options] [argument]", ~[~"test"]);
  // Those are valid options:
  ctx.add_option(None, Some('a'), None, Flags::Defaults).unwrap();
  ctx.add_option(None, Some('a'), None, Flags::Defaults).unwrap_err();
  {
    let (_ ,cmd) = ctx.add_command("command", "description").unwrap();
    cmd.add_option(None, Some('b'), None, Flags::Defaults).unwrap();
    cmd.add_option(None, Some('a'), None, Flags::Defaults).unwrap();
    cmd.add_option(None, Some('b'), None, Flags::Defaults).unwrap_err();
  }
  {
    let (_ ,cmd2) = ctx.add_command("command2", "description").unwrap();
    cmd2.add_option(None, Some('b'), None, Flags::Defaults).unwrap();
    cmd2.add_option(None, Some('a'), None, Flags::Defaults).unwrap();
    cmd2.add_option(None, Some('b'), None, Flags::Defaults).unwrap_err();
  }
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
}

#[test]
fn test_command_valid_passed() {
  let args = ~[~"test", ~"command"];
  let mut ctx = Context::new("test [option] command [command-options]", args);
  // Those are valid options:
  let (cmd_opt, cmd_res) = {
    let (cmd_res ,cmd) = ctx.add_command("command", "description").unwrap();
    (cmd.add_option(None, Some('b'), None, Flags::Defaults).unwrap(), cmd_res)
  };
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  assert!(cmd_res.check());
  assert!(!cmd_opt.check());
}

#[test]
fn test_command_valid_unpassed() {
  let args = ~[~"test", ~"notacommand"];
  let mut ctx = Context::new("test [option] command [command-options]", args);
  // Those are valid options:
  let (cmd_opt, cmd_res) = {
    let (cmd_res ,cmd) = ctx.add_command("command", "description").unwrap();
    (cmd.add_option(None, Some('b'), None, Flags::Defaults).unwrap(), cmd_res)
  };
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});
  assert!(!cmd_res.check());
  assert!(!cmd_opt.check());
}

#[test]
fn test_command_invalid_command_option() {
  let args = ~[~"test", ~"-a", ~"command2", ~"-a"];
  let mut ctx = Context::new("test [option] command [command-options]", args);
  // Those are valid options:
  ctx.add_option(None, Some('a'), None, Flags::Defaults).unwrap();
  ctx.add_option(None, Some('u'), None, Flags::Defaults).unwrap();
  {
    let (_ ,cmd2) = ctx.add_command("command2", "description").unwrap();
    (cmd2.add_option(None, Some('b'), None, Flags::Defaults).unwrap(),
    cmd2.add_option(None, Some('c'), None, Flags::Defaults).unwrap())
  };
  match ctx.validate() {
    Err(msg) => ctx.print_help(Some(msg.as_slice())),
    Ok(()) => assert!(false)
  }
}

#[test]
fn test_command_invalid_command() {
  let args = ~[~"test", ~"-a", ~"command", ~"-b"];
  let mut ctx = Context::new("test [option] command [command-options]", args);
  // Those are valid options:
  ctx.add_option(None, Some('a'), None, Flags::Defaults).unwrap();
  ctx.add_option(None, Some('u'), None, Flags::Defaults).unwrap();
  {
    let (_ ,cmd2) = ctx.add_command("command2", "description").unwrap();
    (cmd2.add_option(None, Some('b'), None, Flags::Defaults).unwrap(),
    cmd2.add_option(None, Some('c'), None, Flags::Defaults).unwrap())
  };
  match ctx.validate() {
    Err(msg) => ctx.print_help(Some(msg.as_slice())),
    Ok(()) => assert!(false)
  }
}

#[test]
fn test_command_option_check_results() {
  let args = ~[~"test", ~"-a", ~"command2", ~"-b"];
  let mut ctx = Context::new("test [option] command [command-options]", args);
  // Those are valid options:
  let a_opt = ctx.add_option(None, Some('a'), None, Flags::Defaults).unwrap();
  let u_opt = ctx.add_option(None, Some('u'), None, Flags::Defaults).unwrap();
  let (cmd_a_opt, cmd_b_opt, cmd_res) = {
    let (cmd_res, cmd) = ctx.add_command("command", "description").unwrap();
    (cmd.add_option(None, Some('a'), None, Flags::Defaults).unwrap(),
    cmd.add_option(None, Some('b'), None, Flags::Defaults).unwrap(),
    cmd_res)
  };

  let (cmd2_b_opt, cmd2_c_opt, cmd2_res) = {
    let (cmd2_res, cmd2) = ctx.add_command("command2", "description").unwrap();
    (cmd2.add_option(None, Some('b'), None, Flags::Defaults).unwrap(),
    cmd2.add_option(None, Some('c'), None, Flags::Defaults).unwrap(),
    cmd2_res)
  };
  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});

  assert!(a_opt.check());
  assert!(!u_opt.check());
  assert!(!cmd_res.check());
  assert!(!cmd_a_opt.check());
  assert!(!cmd_b_opt.check());
  assert!(cmd2_res.check());
  assert!(cmd2_b_opt.check());
  assert!(!cmd2_c_opt.check());
}

#[test]
fn test_command_option_with() {
  let args = ~[~"test", ~"-a", ~"-c", ~"command", ~"-b", ~"-c", ~"cvalue", ~"argument"];
  let mut ctx = Context::new("test [options] command [command-options] [argument]", args);

  // Those are valid options:
  let a_opt = ctx.add_sopt('a', "Option a");
  let b_opt = ctx.add_sopt('b', "Option b");
  let c_opt = ctx.add_sopt('c', "Option c");

  // Those are valid commands:
  let (cmd_res, (cmd_b, cmd_c, cmd_d, cmd_f)) =
    ctx.add_cmd_with("command", "description", |cmd| {
    (cmd.add_sopt('b', "Cmd option b"),
    cmd.add_option(None, Some('c'), Some("Cmd option c"), Flags::TakesArg).unwrap(),
    cmd.add_sopt('d', "Cmd option d"),
    cmd.add_opt("fopt", 'f', "Cmd option f"))
  });

  ctx.validate().map_err(|msg| { ctx.print_help(Some(msg.as_slice())); assert!(false);});

  assert!(a_opt.check());
  assert!(!b_opt.check());
  assert!(c_opt.check());

  assert!(cmd_res.check());
  assert!(cmd_b.check());
  assert!(!cmd_d.check());
  assert!(!cmd_f.check());

  match cmd_c.take_value::<~str>() {
    Ok(Some(val)) => assert!(val == ~"cvalue"), _ => assert!(false),
  }

  assert!(ctx.get_args().as_slice().head().unwrap() == &~"argument");
}

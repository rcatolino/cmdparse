extern mod cmdparse;

use cmdparse::*;

#[test]
fn test_create_option1() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options with no value:
  ctx.create_option(Some("long"), Some("l"), Some("description"),
                    NoValue, Empty).unwrap();
  ctx.create_option(Some("long2"), None, Some("description"),
                    NoValue, Empty).unwrap();
  ctx.create_option(None, Some("r"), Some("description"),
                    NoValue, Empty).unwrap();
  ctx.create_option(Some("long3"), Some("l"), None,
                    NoValue, Empty).unwrap();
}

#[test]
fn test_create_option2() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options which take values but have no defaults
  ctx.create_option(None, Some("a"), None, Str, Empty).unwrap();
  ctx.create_option(None, Some("b"), None, Int, Empty).unwrap();
  ctx.create_option(None, Some("c"), None, Bool, Empty).unwrap();
}

#[test]
fn test_create_option3() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options which take values and have default values.
  ctx.create_option(None, Some("d"), None, Str, StrValue("test")).unwrap();
  ctx.create_option(None, Some("e"), None, Int, IntValue(1)).unwrap();
  ctx.create_option(None, Some("f"), None, Bool, BoolValue(true)).unwrap();
}

#[test]
fn test_create_option4() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are invalid options with no values.
  ctx.create_option(None, Some("g"), None, NoValue, StrValue("test")).unwrap_err();
  ctx.create_option(None, Some("h"), None, NoValue, IntValue(1)).unwrap_err();
  ctx.create_option(None, Some("i"), None, NoValue, BoolValue(true)).unwrap_err();
  ctx.create_option(None, Some("j"), None, NoValue, Present).unwrap_err();
}

#[test]
fn test_create_option5() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are invalid options with str values.
  ctx.create_option(None, Some("k"), None, Str, Present).unwrap_err();
  ctx.create_option(None, Some("l"), None, Str, IntValue(1)).unwrap_err();
  ctx.create_option(None, Some("m"), None, Str, BoolValue(true)).unwrap_err();
}

#[test]
fn test_create_option6() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are invalid options with int values.
  ctx.create_option(None, Some("n"), None, Int, Present).unwrap_err();
  ctx.create_option(None, Some("o"), None, Int, StrValue("test")).unwrap_err();
  ctx.create_option(None, Some("p"), None, Int, BoolValue(true)).unwrap_err();
}

#[test]
fn test_create_option7() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are invalid options with bool values.
  ctx.create_option(None, Some("q"), None, Bool, Present).unwrap_err();
  ctx.create_option(None, Some("r"), None, Bool, IntValue(1)).unwrap_err();
  ctx.create_option(None, Some("s"), None, Bool, StrValue("test")).unwrap_err();
}

#[test]
fn test_create_option8() {
  // Can't have an argument with no value.
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  ctx.create_option(None, None, None, NoValue, Empty).unwrap_err();
}

#[test]
fn test_parse_valid() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  ctx.create_option(Some("long1"), Some("s"), Some("description"),
                    NoValue, Empty);
  ctx.create_option(Some("long2"), None, Some("description"),
                    NoValue, Empty);
  ctx.create_option(None, Some("r"), Some("description"),
                    NoValue, Empty);
  ctx.create_option(Some("help"), Some("h"), None,
                    NoValue, Empty);
  ctx.create_option(None, None, None, NoValue, Empty);

}


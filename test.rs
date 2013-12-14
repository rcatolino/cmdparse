extern mod cmdparse;

use cmdparse::OptContext;
use cmdparse::Flags;

#[test]
fn test_create_option_valid() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options with no value:
  ctx.create_option(Some("long"), Some("l"), Some("description"), Flags::Defaults).unwrap();
  ctx.create_option(Some("long2"), None, Some("description"), Flags::Defaults).unwrap();
  ctx.create_option(Some("long3"), Some("l"), None, Flags::Defaults).unwrap();
  ctx.create_option(Some("long3"), None, None, Flags::Defaults).unwrap();
  ctx.create_option(None, Some("r"), Some("description"), Flags::Defaults).unwrap();
  ctx.create_option(None, Some("a"), None, Flags::Defaults).unwrap();
}

#[test]
fn test_create_option_invalid() {
  let mut ctx = OptContext::new("test [option] [argument]", ~[~"test"]);
  // Those are valid options which take values but have no Flags::Defaults
  ctx.create_option(None, None, None, Flags::Defaults).unwrap_err();
  ctx.create_option(None, None, Some("description"), Flags::Defaults).unwrap_err();
}


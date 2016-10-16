macro_rules! abort {
  ($fmt:expr) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!($fmt, "\n"))
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
  ($fmt:expr, $($arg:tt)*) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!($fmt, "\n"), $($arg)*)
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
}

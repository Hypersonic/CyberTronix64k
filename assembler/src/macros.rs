macro_rules! error {
  ($line:expr, $char:expr, $fmt:expr) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!("error at ({},{}): ", $line, $char)
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!($fmt, "\n"))
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
  ($line:expr, $char:expr, $fmt:expr, $($arg:tt)*) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!("error at ({},{}): ", $line, $char)
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!($fmt, "\n"), $($arg)*)
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
}

// no line number
macro_rules! error_nln {
  ($fmt:expr) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!("error: ", $fmt, "\n"))
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
  ($fmt:expr, $($arg:tt)*) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(
        concat!("error: ", $fmt, "\n"), $($arg)*
      )
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
}

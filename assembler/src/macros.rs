macro_rules! error {
  ($position:expr, $fmt:expr) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!("error at {} -- ", $position)
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!($fmt, "\n"))
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
  ($position:expr, $fmt:expr, $($arg:tt)*) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!("error at {} -- ", $position)
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!($fmt, "\n"), $($arg)*)
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
}

// no line number
macro_rules! error_np {
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

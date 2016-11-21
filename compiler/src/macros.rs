/*
 * macros
 * defines all macros for the rc compiler.
 */

macro_rules! eprint {
  ($fmt:expr) => (
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!($fmt),
    ).expect("Writing to stderr failed")
  );
  ($fmt:expr, $($arg:tt)*) => (
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!($fmt, $($arg)*),
    ).expect("Writing to stderr failed")
  );
}

macro_rules! eprintln {
  ($fmt:expr) => (
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!($fmt, "\n")),
    ).expect("Writing to stderr failed")
  );
  ($fmt:expr, $($arg:tt)*) => (
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!($fmt, "\n"), $($arg)*),
    ).expect("Writing to stderr failed")
  );
}

#[cfg(rcdbg)]
macro_rules! debug {
  ($fmt:expr) => (
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!($fmt, "\n")),
    ).expect("Writing to stderr failed")
  );
  ($fmt:expr, $($arg:tt)*) => (
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!($fmt, "\n"), $($arg)*),
    ).expect("Writing to stderr failed")
  );
}

#[cfg(not(rcdbg))]
macro_rules! debug {
  ($fmt:expr) => ();
  ($fmt:expr, $($arg:tt)*) => (
    let _ = format_args!(concat!($fmt, "\n"), $($arg)*);
  );
}

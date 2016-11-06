macro_rules! error {
  ($position:expr, $fmt:expr) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!("error: ", $fmt))
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(" at {}\n", $position)
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(
        "note: assembler at {}:{}\n",  file!(), line!(),
      ),
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
  ($position:expr, $fmt:expr, $($arg:tt)*) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!("error: ", $fmt), $($arg)*)
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(" at {}\n", $position)
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(
        "note: assembler at {}:{}\n",  file!(), line!(),
      ),
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
}

// no position
macro_rules! error_np {
  ($fmt:expr) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!("error: ", $fmt, "\n"))
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(
        "note: assembler at: {}:{}\n",  file!(), line!(),
      ),
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
  ($fmt:expr, $($arg:tt)*) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(
        concat!("error: ", $fmt, "\n"), $($arg)*
      )
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(
        "note: assembler at: {}:{}\n",  file!(), line!(),
      ),
    ).expect("Writing to stderr failed");
    ::std::process::exit(1);
  });
}

macro_rules! warning {
  ($position:expr, $fmt:expr) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(concat!("warning: ", $fmt)),
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(" at {}\n", $position),
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(
        "note: assembler at {}:{}\n",  file!(), line!(),
      ),
    ).expect("Writing to stderr failed");
  });
  ($position:expr, $fmt:expr, $($arg:tt)*) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(),
      format_args!(concat!("warning: ", $fmt), $($arg)*),
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(" at {}\n", $position),
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(
        "note: assembler at {}:{}\n",  file!(), line!(),
      ),
    ).expect("Writing to stderr failed");
  });
}

// no position
macro_rules! warning_np {
  ($fmt:expr) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(),
      format_args!(concat!("warning: ", $fmt, "\n")),
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(
        "note: assembler at {}:{}\n",  file!(), line!(),
      ),
    ).expect("Writing to stderr failed");
  });
  ($fmt:expr, $($arg:tt)*) => ({
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(
        concat!("warning: ", $fmt, "\n"), $($arg)*,
      )
    ).expect("Writing to stderr failed");
    <::std::io::Stderr as ::std::io::Write>::write_fmt(
      &mut ::std::io::stderr(), format_args!(
        "note: assembler at {}:{}\n",  file!(), line!(),
      ),
    ).expect("Writing to stderr failed");
  });
}

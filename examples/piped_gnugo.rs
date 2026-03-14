use std::{
    io::{self, BufRead, BufReader},
    process::{Command, Stdio},
};

fn main() -> io::Result<()> {
    let child = Command::new("gnugo")
        .arg("--mode=gtp")
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(child) => child,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!("Error: 'gnugo' executable not found.");
            eprintln!("Please install GNU Go and ensure it is in your PATH.");
            eprintln!("Example (Debian/Ubuntu): sudo apt install gnugo");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to start gnugo: {}", e);
            std::process::exit(1);
        }
    };

    println!("gnugo started with pid {}", child.id());

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    'outer: loop {
        let mut response = String::new();

        'inner: loop {
            let mut line = String::new();
            let size = reader.read_line(&mut line)?;
            if size == 0 {
                // EOF
                break 'outer;
            }

            if line.as_str() == "\n" {
                // response end with a empty newline
                break 'inner;
            }

            response.push_str(&line);
        }
        print!("{response}");
    }

    Ok(())
}

#![allow(unused)]

use std::{
    fs,
    io::{self, Write},
    time::Duration,
};

use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode},
    style::{PrintStyledContent, Stylize},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand,
};

use clap::Parser;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn cols() -> u16 {
    terminal::size().expect("Unable to get terminal size").0
}

fn rows() -> u16 {
    terminal::size().expect("Unable to get terminal size").1
}

#[derive(Parser, Debug)]
#[clap(author, version)]
struct Args {
    #[clap(value_parser)]
    filename: String,
    #[clap(short = 'C', long)]
    nocolor: bool,
    #[clap(short = 'U', long, value_parser, default_value_t = 5)]
    update_time: u64,
}

struct State {
    color: bool,
    update_time: Duration,
    stdout: io::Stdout,
    buf: String,
}
impl State {
    fn new(args: &Args) -> Self {
        Self {
            color: !args.nocolor,
            update_time: Duration::from_millis(args.update_time),
            stdout: io::stdout(),
            buf: fs::read_to_string(&args.filename)
                .expect(format!("Could not open file '{}'", args.filename).as_str()),
        }
    }

    fn init(&mut self) -> crossterm::Result<()> {
        self.exec(EnterAlternateScreen)?.exec(cursor::Hide)?;
        terminal::enable_raw_mode()?;
        Ok(())
    }

    fn deinit(&mut self) -> crossterm::Result<()> {
        self.exec(LeaveAlternateScreen)?.exec(cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn get_input(&self) -> crossterm::Result<bool> {
        if event::poll(self.update_time)? {
            match event::read()? {
                Event::Key(e) => match e.code {
                    KeyCode::Char(c) => match c {
                        'q' => return Ok(true),
                        _ => (),
                    },
                    KeyCode::Esc => return Ok(true),
                    _ => (),
                },
                _ => (),
            }
        }
        Ok(false)
    }

    fn display(&mut self) -> crossterm::Result<()> {
        let color = self.color;

        let mut line = 0;

        self.queue(MoveTo(0, rows() - 1))?
            .queue(PrintStyledContent(if color {
                format!("grm v{} | ? for help", VERSION).black().on_green()
            } else {
                format!("grm v{} | ? for help", VERSION).stylize()
            }))?
            .queue(MoveTo(0, 0))?;

        for ch in self.buf.clone().chars() {
            match ch {
                '\n' => {
                    line += 1;
                    self.queue(MoveTo(0, line))?;
                }
                _ => putchar(&mut self.stdout, ch)?,
            }
        }
        Ok(())
    }

    fn exec(&mut self, cmd: impl crossterm::Command) -> crossterm::Result<&mut Self> {
        self.stdout.execute(cmd)?;
        Ok(self)
    }

    fn queue(&mut self, cmd: impl crossterm::Command) -> crossterm::Result<&mut Self> {
        self.stdout.queue(cmd)?;
        Ok(self)
    }
}

fn putchar(stdout: &mut io::Stdout, ch: char) -> io::Result<()> {
    let mut buf = [0; 4];
    ch.encode_utf8(&mut buf);
    stdout.write(&buf)?;
    Ok(())
}

fn main() -> crossterm::Result<()> {
    let mut state = State::new(&Args::parse());
    state.init()?;

    loop {
        if state.get_input()? {
            break;
        }
        state.display()?;
    }

    state.deinit()
}

use clap::Parser;
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};
use toml::Value;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]

struct Cli {
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    #[arg(short, long, default_value = "toml")]
    pub output: Format,

    #[arg(short, long, default_value = "toml")]
    pub input: Format,

    #[arg(short, long)]
    pub preety: bool,

    pub pattern: String,

    #[cfg(feature = "syntax-highlighting")]
    #[arg(short, long, default_value = "auto")]
    pub color: clap::ColorChoice,
}

#[derive(Default, Debug, Copy, Clone, clap::ValueEnum)]
enum Format {
    #[default]
    Toml,

    #[cfg(feature = "json")]
    Json,
}

fn main() -> anyhow::Result<()> {
    let app = Cli::parse();

    #[cfg(feature = "syntax-highlighting")]
    match app.color {
        clap::ColorChoice::Auto => {}
        clap::ColorChoice::Never => console::set_colors_enabled(false),
        clap::ColorChoice::Always => console::set_colors_enabled(true),
    }

    let mut reader: Box<dyn Read> = match &app.file {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(io::stdin()),
    };

    let mut input_string = String::new();
    reader.read_to_string(&mut input_string)?;

    let input_string = match app.input {
        Format::Toml => input_string,
        #[cfg(feature = "json")]
        Format::Json => {
            if let Ok(json_value) = serde_json::from_str::<toml::Value>(&input_string) {
                toml::to_string(&json_value)?
            } else {
                input_string
            }
        }
    };

    let toml_value: toml::Value = toml::from_str(&input_string)?;

    let result: &Value = to::extract_pattern(&toml_value, &app.pattern)?;

    let output = match (app.output, app.preety) {
        (Format::Toml, false) => toml::to_string(result)?,
        (Format::Toml, true) => toml::to_string_pretty(result)?,

        #[cfg(feature = "json")]
        (Format::Json, false) => serde_json::to_string(result)?,

        #[cfg(feature = "json")]
        (Format::Json, true) => serde_json::to_string_pretty(result)?,
    };

    #[cfg(feature = "syntax-highlighting")]
    {
        let mut pretty_printer = bat::PrettyPrinter::new();

        pretty_printer
            .colored_output(console::colors_enabled())
            .grid(false)
            .rule(false)
            .line_numbers(false);

        match app.output {
            Format::Toml => {
                pretty_printer
                    .language("toml")
                    .input_from_bytes(output.as_bytes())
                    .print()?;
            }

            #[cfg(feature = "json")]
            Format::Json => {
                pretty_printer
                    .language("json")
                    .input_from_bytes(output.as_bytes())
                    .print()?;
            }
        }
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    println!("{output}");

    Ok(())
}

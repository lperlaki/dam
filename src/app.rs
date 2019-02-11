use clap::{App, Arg, SubCommand};
use std::path::PathBuf;

pub fn build() -> App<'static, 'static> {
    App::new("Open DAM")
        .version("0.1.1")
        .author("Liam P. <lperlaki@icloud.com>")
        .about("Digital Asset Manager")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .default_value(".")
                .validator(|val| {
                    let path = PathBuf::from(val);
                    if path.exists() && path.is_dir() {
                        Ok(())
                    } else {
                        Err(String::from("Must be a valid directory!"))
                    }
                })
                .index(1),
        )
}

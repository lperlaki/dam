use clap::{App, AppSettings, Arg, SubCommand};
use std::path::PathBuf;

pub fn build() -> App<'static, 'static> {
    App::new("Open DAM")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .author("Liam P. <lperlaki@icloud.com>")
        .about("Digital Asset Manager")
        .arg(
            Arg::with_name("DIR")
                .short("d")
                .long("dir")
                .help("Sets the DAM home dir")
                .global(true)
                .takes_value(true)
                .default_value_os(std::path::Component::CurDir.as_os_str())
                .validator(|val| {
                    let path = PathBuf::from(val);
                    if path.exists() && path.is_dir() {
                        Ok(())
                    } else {
                        Err(String::from("Must be a valid directory!"))
                    }
                }),
        )
        .subcommand(SubCommand::with_name("init").about("init folder as dam"))
        .subcommand(SubCommand::with_name("list").about("list all files"))
        .subcommand(SubCommand::with_name("scan").about("scan for new"))
        .subcommand(
            SubCommand::with_name("open")
                .about("open for new")
                .arg(Arg::with_name("NAME").required(true)),
        )
}

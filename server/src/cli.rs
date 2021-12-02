use clap::{App, Arg, SubCommand};



pub fn build_app() -> App<'static,'static> {
    App::new("home-radio")
        .version("1.0.0")
        .author("Rene Richter")
        .subcommand(
            SubCommand::with_name("serve")
                    .arg(
                        Arg::with_name("autoplay")
                            .long("autoplay")
                            .takes_value(false)
                    )
                    .arg(
                        Arg::with_name("dir")
                            .long("dir")
                            .takes_value(true)
                            .default_value("/var/lib/home-radio")
                    )
        )
}
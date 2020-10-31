const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

pub fn build_cli() -> clap::App<'static, 'static> {
    clap_app!( app_config =>
        (version: VERSION)
        (name: NAME)
        (about: "app_config: watch AWS appConfig for changes and take action")
        (@subcommand check =>
            (about: "Look for Updates")
            (@arg FILE: -f --file +takes_value +required)
        )
        (@subcommand query =>
            (about: "Print last data received")
            (@arg FILE: -f --file +takes_value +required)
        )
        (@subcommand params =>
            (about: "Get Parameters")
        )
        (@subcommand bash =>
            (about: "Generate a bash autocompletion script")
        )
    )
}

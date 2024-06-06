use clap::CommandFactory;

#[derive(clap::Parser)]
struct Cli {
    #[clap(long)]
    arg1: String,
    /// The description for "arg2"
    #[clap(long, default_value = "my-default-value")]
    arg2: String,
    #[clap(long)]
    my_flag: bool,
    #[clap(long)]
    my_vec: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The env var GENERATE_ICINGA_COMMAND has to be set to generate the Icinga command configuration
    nagiosplugin::config_generator::print_icinga_command_config_if_env_and_exit(
        "example",
        &Cli::command(),
    )?;

    println!("Set the environment variable GENERATE_ICINGA_COMMAND to generate the Icinga command configuration.");

    Ok(())
}

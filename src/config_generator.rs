pub struct CommandDescription {
    arguments: Vec<ArgumentDescription>,
}

pub struct ArgumentDescription {
    name: String,
    value: String,
    description: Option<String>,
    is_flag: bool,
    default_value: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ToIcingaCommandError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid executable path")]
    InvalidExecutablePath,
    #[error("error converting to command description: {0}")]
    CommandDescriptionFromError(#[from] CommandDescriptionFromError),
}

impl CommandDescription {
    pub fn to_icinga_command(&self, name: &str) -> Result<String, ToIcingaCommandError> {
        let mut out = format!("object CheckCommand \"{name}\" {{\n");
        let current_exe = std::env::current_exe()?
            .to_str()
            .ok_or(ToIcingaCommandError::InvalidExecutablePath)?
            .to_owned();

        out.push_str(&format!("  command = [ \"{current_exe}\" ]\n"));
        out.push_str("  arguments = {\n");
        for arg in &self.arguments {
            out.push_str(&format!("  \"{}\" = {{\n", arg.name));

            if arg.is_flag {
                out.push_str(&format!("    set_if = \"${}$\"\n", arg.value));
            } else {
                out.push_str(&format!("    value = \"${}$\"\n", arg.value));
            }

            if let Some(description) = &arg.description {
                out.push_str(&format!(
                    "    description = \"{}\"\n",
                    escape_string(description)
                ));
            }

            out.push_str("  }\n");
        }

        out.push_str("\n");

        for arg in &self.arguments {
            if let Some(default_value) = &arg.default_value {
                out.push_str(&format!(
                    "  vars.{} = \"{}\"\n",
                    arg.value,
                    escape_string(default_value)
                ));
            }
        }

        out.push_str("}\n");
        Ok(out)
    }
}

fn escape_string(s: &str) -> String {
    ["\"", "$"]
        .iter()
        .fold(s.to_string(), |acc, c| acc.replace(c, &format!("\\{}", c)))
}

#[derive(Debug, thiserror::Error)]
pub enum CommandDescriptionFromError {
    #[error("missing long argument")]
    MissingLongArgument,
}

impl TryFrom<&clap::Command> for CommandDescription {
    type Error = CommandDescriptionFromError;

    fn try_from(cmd: &clap::Command) -> Result<Self, Self::Error> {
        let mut arguments = Vec::new();

        for arg in cmd.get_arguments() {
            let name = arg
                .get_long()
                .ok_or(CommandDescriptionFromError::MissingLongArgument)?
                .to_owned();

            let value = name.replace("-", "_");
            let description = arg.get_help().map(|s| s.to_string());

            let is_flag = {
                let values = arg.get_possible_values();
                values.len() == 2
                    && values.iter().find(|v| v.get_name() == "true").is_some()
                    && values.iter().find(|v| v.get_name() == "false").is_some()
            };

            let default_value = arg
                .get_default_values()
                .first()
                .and_then(|v| v.to_str())
                .map(|s| s.to_string());

            arguments.push(ArgumentDescription {
                name,
                value,
                description,
                is_flag,
                default_value,
            });
        }

        Ok(CommandDescription { arguments })
    }
}

/// Print the Icinga command configuration if the GENERATE_ICINGA_COMMAND environment variable is set
/// and exit the process.
pub fn print_icinga_command_config_if_env_and_exit(
    name: &str,
    cmd: &clap::Command,
) -> Result<(), ToIcingaCommandError> {
    if !std::env::var("GENERATE_ICINGA_COMMAND").is_ok() {
        return Ok(());
    }

    let description = CommandDescription::try_from(cmd)?;
    let out = description.to_icinga_command(name)?;

    println!("{}", out.trim());
    std::process::exit(0);
}

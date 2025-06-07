use clap::Parser;
use serde::Deserialize;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::process::{Command, Stdio};

#[derive(Deserialize, Debug)]
struct CSRConfig {
    country: String,
    state_or_province: String,
    locality: String,
    org_name: String,
    common_name: String,
    email: Option<String>,
    password: Option<String>,
}

#[derive(Debug)]
enum CSRConfigConversionError {
    InvalidArguments,
}

impl Display for CSRConfigConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for CSRConfigConversionError {}

impl TryFrom<CSRConfigCommand> for CSRConfig {
    type Error = CSRConfigConversionError;
    fn try_from(value: CSRConfigCommand) -> Result<Self, Self::Error> {
        let country = value
            .country
            .ok_or(CSRConfigConversionError::InvalidArguments)?;
        let state_or_province = value
            .state_or_province
            .ok_or(CSRConfigConversionError::InvalidArguments)?;
        let locality = value
            .locality
            .ok_or(CSRConfigConversionError::InvalidArguments)?;
        let org_name = value
            .org_name
            .ok_or(CSRConfigConversionError::InvalidArguments)?;
        let common_name = value
            .common_name
            .ok_or(CSRConfigConversionError::InvalidArguments)?;
        let email = value.email;
        let password = value.password;
        Ok(CSRConfig {
            country,
            state_or_province,
            locality,
            org_name,
            common_name,
            email,
            password,
        })
    }
}

#[derive(Parser, Debug)]
struct CSRConfigCommand {
    #[arg(long)]
    from_file: Option<String>,
    #[arg(long)]
    country: Option<String>,
    #[arg(long)]
    state_or_province: Option<String>,
    #[arg(long)]
    locality: Option<String>,
    #[arg(long)]
    org_name: Option<String>,
    #[arg(long)]
    common_name: Option<String>,
    #[arg(long)]
    email: Option<String>,
    #[arg(long)]
    password: Option<String>,
}

impl ToString for CSRConfig {
    fn to_string(&self) -> String {
        format!(
            "/C={}/ST={}/L={}/O={}/OU={}/CN={}/emailAddress={}",
            self.country,
            self.state_or_province,
            self.locality,
            self.org_name,
            self.common_name,
            self.email.clone().unwrap_or_default(),
            self.password.clone().unwrap_or_default(),
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let csr_config_command = CSRConfigCommand::parse();

    let csr_config: CSRConfig = match &csr_config_command.from_file {
        Some(file_path) => {
            let file = File::open(file_path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader)?
        }
        None => csr_config_command.try_into()?,
    };

    let private_key_path = format!("{}.key", &csr_config.common_name);
    let csr_path = format!("{}.csr", &csr_config.common_name);

    let mut child = Command::new("openssl")
        .args(&[
            "req",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-keyout",
            private_key_path.as_ref(),
            "-out",
            csr_path.as_ref(),
            "-subj",
            csr_config.to_string().as_ref(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let status = child.wait()?;
    println!("Process exited with: {}", status);

    Ok(())
}

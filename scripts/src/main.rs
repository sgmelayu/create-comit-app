#![type_length_limit = "10591395"]

use structopt::StructOpt;

use comit_scripts::{create_comit_app::CreateComitApp, env};

fn main() -> std::io::Result<()> {
    let mut runtime = tokio_compat::runtime::Runtime::new()?;

    let command = CreateComitApp::from_args();

    runtime.block_on_std(run_command(command))?;

    Ok(())
}

async fn run_command(command: CreateComitApp) -> std::io::Result<()> {
    match command {
        CreateComitApp::StartEnv => env::start().await,
        CreateComitApp::ForceCleanEnv => env::clean_up().await,
    }

    Ok(())
}

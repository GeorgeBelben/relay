use clap::{Parser, Subcommand};
use colored::Colorize;
use relay_protocol::{Request, Response};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/tmp/relay.sock";

#[derive(Parser)]
#[command(name = "relay", about = "Talk to the relay-core daemon")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    // Check if the daemon is alive
    Ping,
    // Ask the daemon what's currently running
    State,
    // Launch a rom by an absolute path
    Launch {
        game_id: i64,
    },
    Stop,
    Library,
    Settings {
        #[command(subcommand)]
        action: SettingsAction,
    },
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    Ra {
        #[command(subcommand)]
        action: RaAction,
    },
}

#[derive(Subcommand)]
enum SettingsAction {
    List {
        #[arg(long)]
        system: Option<String>,
    },
    Set {
        #[arg(long)]
        system: Option<String>,
        key: String,
        value: String,
    },
}

#[derive(Subcommand)]
enum ProfileAction {
    Create { name: String },
    List,
    Use { profile_id: i64 },
}

#[derive(Subcommand)]
enum RaAction {
    Link {
        username: String,
        web_api_key: String,
    },
    Stats,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let request = match cli.command {
        Command::Ping => Request::Ping,
        Command::State => Request::GetState,
        Command::Launch { game_id } => Request::LaunchGame { game_id },
        Command::Stop => Request::StopGame,
        Command::Library => Request::GetLibrary,
        Command::Settings { action } => match action {
            SettingsAction::List { system } => Request::GetSettings { system },
            SettingsAction::Set { system, key, value } => {
                Request::SetSetting { system, key, value }
            }
        },
        Command::Profile { action } => match action {
            ProfileAction::Create { name } => Request::CreateProfile { name },
            ProfileAction::List => Request::ListProfiles,
            ProfileAction::Use { profile_id } => Request::SetActiveProfile { profile_id },
        },
        Command::Ra { action } => match action {
            RaAction::Link {
                username,
                web_api_key,
            } => Request::LinkRetroAchievements {
                username,
                web_api_key,
            },
            RaAction::Stats => Request::GetRaStats,
        },
    };

    match send(request).await {
        Ok(response) => print_response(response),
        Err(e) => {
            eprintln!("error {e}");
            std::process::exit(1);
        }
    }
}

async fn send(request: Request) -> Result<Response, String> {
    let stream = UnixStream::connect(SOCKET_PATH)
        .await
        .map_err(|e| format!("couldn't connect to daemon: {e}"))?;

    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    let mut json = serde_json::to_string(&request).unwrap();
    json.push('\n');
    write_half
        .write_all(json.as_bytes())
        .await
        .map_err(|e| format!("write failed: {e}"))?;

    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .map_err(|e| format!("read failed: {e}"))?;

    serde_json::from_str(line.trim()).map_err(|e| format!("bad response: {e}"))
}

fn print_response(response: Response) {
    match response {
        Response::Pong => println!("{}", "Pong".green()),
        Response::Error { message } => {
            eprintln!("{} {message}", "daemon error:".red().bold());
            std::process::exit(1);
        }
        Response::GameLaunched { pid } => println!("{} pid {pid}", "launched,".green()),
        Response::GameExited { exit_code } => println!("game exited, code {exit_code:?}"),
        Response::GameStopped => println!("{}", "game stopped".green()),
        Response::State(state) => match state.running_game {
            Some(game) => println!(
                "{} {} (pid {})",
                "running:".green(),
                game.rom_path,
                game.pid
            ),
            None => println!("{}", "nothing running".dimmed()),
        },
        Response::Library(entries) => {
            if entries.is_empty() {
                println!("library is empty")
            } else {
                for entry in entries {
                    let last_played = match entry.last_played_at {
                        Some(ts) => format!("last played {ts}"),
                        None => "never played".to_string(),
                    };
                    println!(
                        "[{}] ({}) {} — {}s played, {}",
                        entry.system_display_name,
                        entry.id,
                        entry.file_name,
                        entry.play_time_seconds,
                        last_played
                    );
                }
            }
        }
        Response::Settings(entries) => {
            if entries.is_empty() {
                println!("no settings set")
            } else {
                for (key, value) in entries {
                    println!("{key} = {value}");
                }
            }
        }
        Response::SettingSet => println!("setting saved"),
        Response::Profile(profile) => println!(
            "{} {} (id {}){}",
            "profile:".green(),
            profile.name,
            profile.id,
            if profile.ra_linked {
                " [RA linked]"
            } else {
                ""
            }
        ),
        Response::Profiles(profiles) => {
            if profiles.is_empty() {
                println!("no profiles yet")
            } else {
                for profile in profiles {
                    println!(
                        "({}) {}{}",
                        profile.id,
                        profile.name,
                        if profile.ra_linked {
                            " [RA linked]"
                        } else {
                            ""
                        }
                    );
                }
            }
        }
        Response::ActiveProfile(profile) => match profile {
            Some(profile) => println!("{} {} (id {})", "active:".green(), profile.name, profile.id),
            None => println!("{}", "no active profile".dimmed()),
        },
        Response::RaStats {
            points,
            softcore_points,
        } => println!(
            "{} {points} hardcore, {softcore_points} softcore",
            "points:".green()
        ),
        Response::Controllers(controllers) => {
            if controllers.is_empty() {
                println!("no controllers connected")
            } else {
                for c in controllers {
                    println!("({}) {} [{:?}]", c.index, c.name, c.controller_type);
                }
            }
        }
    }
}

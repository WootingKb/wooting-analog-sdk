use chrono::Utc;
use clap::{Parser, arg};
use json::object;
use log::{debug, error, info, warn};
use self_update::update::{Release, ReleaseAsset};
use self_update::version::bump_is_greater;
use simplelog::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
#[cfg(windows)]
use winapi::um::winuser;

const INSTALLER_PATH: &str = "wooting_analog_sdk_installer.msi";
const PKG_VER: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Parser, Debug)]
#[command(name = "Wooting Analog SDK Updater", version, about)]
struct Cli {
    #[arg(long = "no-install")]
    no_install: bool,

    /// Doesn't prompt the user
    #[arg(long = "quiet")]
    quiet: bool,

    /// Additional logging to console (-v, -vv, -vvv)
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    verbose: u8,
    /*
    /// Sets the MSI installer to use
    #[arg(help = "Sets the MSI installer to use", required = true, index = 1)]
    msi: String,
    */
}

fn main() {
    let cli = Cli::parse();
    let mut log_path = PathBuf::new();
    let log_dir = std::env::var("APPDATA")
        .map(|appdata| appdata + "\\wooting-analog-sdk\\")
        .map_err(|e| error!("Unable to get Appdata directory: {}", e))
        .unwrap_or("./".to_string());

    std::fs::create_dir_all(&log_dir).unwrap();
    log_path.push(log_dir);
    log_path.push("updater.log");

    let (term_filter, color) = match cli.verbose {
        0 => (LevelFilter::Off, ColorChoice::Never),
        1 => (LevelFilter::Info, ColorChoice::Auto),
        2 | _ => (LevelFilter::Debug, ColorChoice::Auto),
    };

    CombinedLogger::init(vec![
        TermLogger::new(term_filter, Config::default(), TerminalMode::Mixed, color),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(&log_path)
                .unwrap(),
        ),
    ])
    .unwrap();
    info!(
        "Wooting Analog SDK Updater v{} '{}'",
        PKG_VER,
        Utc::now().format("%a %b %e %T %Y")
    );
    info!("Logging output to: '{:?}'", log_path);

    debug!("Called with parameters {:?}", cli);

    match check_for_update() {
        Ok(release) => {
            debug!(
                "Github release: {} ours: {}, update available",
                release.version, PKG_VER
            );

            let data = object! {
                "name" => "Wooting Analog SDK",
                "update_available" => true,
                "new_version"    => release.version.clone(),
                "version"     => PKG_VER,
                "release_title" => release.name.clone(),
                "release_notes" => release.body.clone()
            };
            println!("{}", data.dump());

            info!("Update available!");
            if !cli.no_install {
                #[cfg(windows)]
                {
                    if !cli.quiet {
                        let title = "Wooting Analog SDK Update\0";
                        let message = format!(
                            "A new Wooting Analog SDK update is available ({}, you've got v{}), would you like to install?\0",
                            release.version, PKG_VER
                        );
                        let l_msg: Vec<u16> = message.encode_utf16().collect();
                        let l_title: Vec<u16> = title.encode_utf16().collect();
                        unsafe {
                            use std::ptr::null_mut;

                            if winuser::MessageBoxW(
                                null_mut(),
                                l_msg.as_ptr(),
                                l_title.as_ptr(),
                                winuser::MB_YESNO | winuser::MB_ICONQUESTION,
                            ) != winuser::IDYES
                            {
                                debug!("User did not want update, closing");
                                return;
                            }
                        }
                    }
                }
                info!("Attempting to update");
                install_update(&release).expect("Failed to install updates");
            } else {
                info!("--no-install given, Exiting without updating...");
            }
        }
        Err(e) => {
            info!("No update available, Exiting...");
            warn!("context: {e}");
        }
    }
}

fn find_installer_asset(release: &Release) -> Option<&ReleaseAsset> {
    release
        .assets
        .iter()
        .find(|asset| asset.name.starts_with("wooting_analog_sdk") && asset.name.ends_with(".msi"))
}

fn check_for_update() -> Result<Release, Box<dyn ::std::error::Error>> {
    if is_stable(PKG_VER) {
        // always grabs the latest release, ignoring pre-releases and drafts, without filtering or
        // version checks
        let latest_release = self_update::backends::github::Update::configure()
            .repo_owner("WootingKb")
            .repo_name("wooting-analog-sdk")
            .bin_name(PKG_NAME)
            .current_version(PKG_VER)
            .build()?
            .get_latest_release()?;

        if bump_is_greater(PKG_VER, &latest_release.version).unwrap_or(false) {
            Ok(latest_release)
        } else {
            Err(Box::from("Already on latest stable release..."))
        }
    } else {
        // grabs the latest releases and filters based on our current version, includes pre-releases
        let releases = self_update::backends::github::Update::configure()
            .repo_owner("WootingKb")
            .repo_name("wooting-analog-sdk")
            .bin_name(PKG_NAME)
            .current_version(PKG_VER)
            .build()?
            .get_latest_releases(PKG_VER)?;

        releases
            .first()
            .cloned()
            .ok_or_else(|| Box::from("Already on latest release..."))
    }
}

fn install_update(release: &Release) -> Result<(), Box<dyn ::std::error::Error>> {
    info!("installing");
    match find_installer_asset(release) {
        Some(asset) => {
            let tmp_dir = self_update::TempDir::new()?;
            let tmp_msi_path = tmp_dir.path().join(INSTALLER_PATH);
            info!("Downloading {:?} into temp file: {:?}", asset, tmp_msi_path);
            //Put it into lower scope to force File to go out of scope to close it & finish writing
            {
                let tmp_msi = ::std::fs::File::create(&tmp_msi_path)?;

                self_update::Download::from_url(&asset.download_url)
                    .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse()?)
                    .show_progress(true)
                    .download_to(&tmp_msi)?;
                info!("Finished downloading update");
            }

            let tmp_install_script_path = tmp_dir.path().join("install.ps1");
            {
                let mut tmp_install_script = ::std::fs::File::create(&tmp_install_script_path)?;
                tmp_install_script.write_all(include_bytes!("install.ps1"))?;
                info!("Finished writing install script");
            }
            info!("Running powershell install script and exiting");
            Command::new("powershell")
                .arg("-ExecutionPolicy")
                .arg("RemoteSigned")
                .arg("-File")
                .arg(tmp_install_script_path.as_os_str())
                .arg("-msi_path")
                .arg(tmp_msi_path.as_os_str())
                .spawn()?;
            //We got to exit like this to stop the tmpdir from being deleted on close (as it is still needed) and so that the updater can be overwrriten by the installer
            std::process::exit(0);
            //Ok(())
        }
        None => Err(From::from("Couldn't find installer asset")),
    }
}

fn is_stable(version: &str) -> bool {
    semver::Version::parse(version)
        .inspect_err(|e| warn!("invalid semver tag '{version}': {e}"))
        .map(|v| v.pre.is_empty())
        .unwrap_or(false)
}

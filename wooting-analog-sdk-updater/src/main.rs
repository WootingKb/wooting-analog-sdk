extern crate clap;
extern crate self_update;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate winapi;
#[macro_use]
extern crate json;
extern crate chrono;

use chrono::Utc;
use clap::{App, Arg};
use self_update::update::{Release, ReleaseAsset};
use self_update::version::bump_is_greater;
use simplelog::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
#[cfg(windows)]
use winapi::um::winuser;

const INSTALLER_PATH: &str = "wooting_analog_sdk_installer.msi";
const PKG_VER: &str = env!("CARGO_PKG_VERSION");

fn find_installer_asset(release: &Release) -> Option<&ReleaseAsset> {
    release
        .assets
        .iter()
        .find(|asset| asset.name.starts_with("wooting_analog_sdk") && asset.name.ends_with(".msi"))
}

fn check_for_update() -> Result<Release, Box<dyn ::std::error::Error>> {
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("WootingKb")
        .repo_name("wooting-analog-sdk")
        .build()?
        .fetch()?;
    //Remove all releases that are not newer than the current
    debug!("We found {:?}", releases);
    if !releases.is_empty() {
        let latest = releases.first().unwrap();
        Ok(latest.clone())
    } else {
        warn!("No releases found on github");
        Err(From::from("Couldn't find any releases on github"))
    }
}

fn install_update(release: &Release) -> Result<(), Box<dyn ::std::error::Error>> {
    info!("installing");
    match find_installer_asset(&release) {
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

use std::path::PathBuf;

fn main() {
    let mut log_path = PathBuf::new();
    let log_dir = std::env::var("APPDATA")
        .map(|appdata| appdata + "\\wooting-analog-sdk\\")
        .map_err(|e| error!("Unable to get Appdata directory: {}", e))
        .unwrap_or("./".to_string());
    std::fs::create_dir_all(&log_dir).unwrap();
    log_path.push(log_dir);
    log_path.push("updater.log");

    let matches = App::new(format!("Wooting Analog SDK Updater v{}", PKG_VER))
        .arg(
            Arg::with_name("no_install")
                .long("no-install")
                .help("Only checks for updates, doesn't install them"),
        )
        .arg(
            Arg::with_name("quiet")
                .long("quiet")
                .help("Doesn't prompt the user"),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Additional logging to console"),
        )
        /*.arg(
            Arg::with_name("MSI")
                .help("Sets the MSI installer to use")
                .required(true)
                .index(1),
        )*/
        .get_matches();
    let term_filter = match matches.occurrences_of("v") {
        0 => LevelFilter::Off,
        1 => LevelFilter::Info,
        2 | _ => LevelFilter::Debug,
    };

    CombinedLogger::init(vec![
        TermLogger::new(term_filter, Config::default(), TerminalMode::Mixed).unwrap(),
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

    debug!("Called with parameters {:?}", matches);

    let r = check_for_update().expect("Failed to check for updates");
    let release_ver = r.version.trim_start_matches('v');
    let update_available = bump_is_greater(PKG_VER, release_ver).unwrap_or(false);
    debug!(
        "Github release: {} ours: {}, update available: {}",
        release_ver, PKG_VER, update_available
    );

    let data = object! {
        "name" => "Wooting Analog SDK",
        "update_available" => update_available,
        "new_version"    => r.version.clone(),
        "version"     => PKG_VER,
        "release_title" => r.name.clone(),
        "release_notes" => r.body.clone()
    };
    println!("{}", data.dump());

    if update_available {
        info!("Update available!");
        if !matches.is_present("no_install") {
            #[cfg(windows)]
            {
                if !matches.is_present("quiet") {
                    let title = "Wooting Analog SDK Update\0";
                    let message = format!("A new Wooting Analog SDK update is available ({}, you've got v{}), would you like to install?\0", r.version, PKG_VER);
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
            install_update(&r).expect("Failed to install updates");
        } else {
            info!("--no-install given, Exiting without updating...");
        }
    } else {
        info!("No update available, Exiting...");
    }
}

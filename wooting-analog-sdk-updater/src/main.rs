extern crate clap;
extern crate self_update;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate winapi;

use std::ptr::null_mut as NULL;
#[cfg(windows)]
use winapi::um::winuser;
use clap::{App, Arg};
use env_logger::Env;
use self_update::backends::github::{Release, ReleaseAsset};
use self_update::version::bump_is_greater;
use std::io::Write;
use std::process::Command;

const INSTALLER_PATH: &str = "installer.msi";
const PKG_VER: &str = env!("CARGO_PKG_VERSION");

fn find_installer_asset(release: &Release) -> Option<&ReleaseAsset> {
    release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(".msi"))
}

fn check_for_update() -> Result<Option<Release>, Box<dyn ::std::error::Error>> {
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("WootingKb")
        .repo_name("wooting-analog-sdk")
        .build()?
        .fetch()?;
    //Remove all releases that are not newer than the current
    info!("We found {:?}", releases);
    if !releases.is_empty() {
        let latest = releases.first().unwrap();
        let release_ver = latest.tag.trim_start_matches('v');
        debug!("Latest Github release has got version: {}", release_ver);
        if bump_is_greater(PKG_VER, release_ver).unwrap_or(false) {
            debug!(
                "Github release {} is greater than our {} so let's update",
                release_ver,
                PKG_VER
            );
            if find_installer_asset(latest).is_some() {
                Ok(Some(latest.clone()))
            } else {
                error!("We found latest release with ver {}, but we can't find an msi installer for it", release_ver);
                Ok(None)
            }
        } else {
            debug!(
                "Github release {} is not greater than our {} so let's chill",
                release_ver,
                PKG_VER
            );
            Ok(None)
        }
    } else {
        warn!("No releases found on github");
        Ok(None)
    }
}

fn install_update(release: Release) -> Result<(), Box<dyn ::std::error::Error>> {
    info!("installing");
    match find_installer_asset(&release) {
        Some(asset) => {
            let tmp_dir = self_update::TempDir::new("wooting_analog_sdk_updater")?;
            let tmp_msi_path = tmp_dir.path().join(INSTALLER_PATH);
            debug!("Downloading into temp file: {:?}", tmp_msi_path);
            //Put it into lower scope to force File to go out of scope to close it & finish writing
            {
                let tmp_msi = ::std::fs::File::create(&tmp_msi_path)?;

                self_update::Download::from_url(&asset.download_url).download_to(&tmp_msi)?;
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

fn main() {
    let matches = App::new("Wooting Analog SDK Updater")
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
        /*.arg(
            Arg::with_name("MSI")
                .help("Sets the MSI installer to use")
                .required(true)
                .index(1),
        )*/
        .get_matches();
    env_logger::from_env(Env::default().default_filter_or("wooting_analog_sdk_updater=trace")).init();

    let release = check_for_update().expect("Failed to check for updates");
    match release {
        Some(r) => {
            info!("New version found: {}", r.version());
            if !matches.is_present("no_install") {
                #[cfg(windows)]
                {
                    if !matches.is_present("quiet") {
                        let title = "Wooting Analog SDK Update\0";
                        let message = format!("A new Wooting Analog SDK update is available ({}, you've got v{}), would you like to install?\0", r.tag, PKG_VER);
                        let l_msg: Vec<u16> = message.encode_utf16().collect();
                        let l_title: Vec<u16> = title.encode_utf16().collect();
                        unsafe {
                            if winuser::MessageBoxW(NULL(), l_msg.as_ptr(), l_title.as_ptr(), winuser::MB_YESNO | winuser::MB_ICONQUESTION) != winuser::IDYES {
                                info!("User did not want update, closing");
                                return;
                            }
                        }
                    }
                }

                install_update(r).expect("Failed to install updates");
            }
        }
        None => {
            info!("You are using the latest version");
        }
    }

    //Install this process as a scheduled task that runs every so often
}

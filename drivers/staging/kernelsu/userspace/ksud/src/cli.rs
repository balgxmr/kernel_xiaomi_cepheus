use anyhow::{Ok, Result};
use clap::Parser;
use std::path::PathBuf;

#[cfg(target_os = "android")]
use android_logger::Config;
#[cfg(target_os = "android")]
use log::LevelFilter;

use crate::{apk_sign, debug, defs, event, module, server, utils};

/// KernelSU userspace cli
#[derive(Parser, Debug)]
#[command(author, version = defs::VERSION_NAME, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Manage KernelSU modules
    Module {
        #[command(subcommand)]
        command: Module,
    },

    /// Trigger `post-fs-data` event
    PostFsData,

    /// Trigger `service` event
    Services,

    /// Trigger `boot-complete` event
    BootCompleted,

    /// Install KernelSU userspace component to system
    Install,

    /// SELinux policy Patch tool
    Sepolicy {
        #[command(subcommand)]
        command: Sepolicy,
    },

    /// Manage App Profiles
    Profile {
        #[command(subcommand)]
        command: Profile,
    },

    /// Patch boot or init_boot images to apply KernelSU
    BootPatch {
        /// boot image path, if not specified, will try to find the boot image automatically
        #[arg(short, long)]
        boot: Option<PathBuf>,

        /// kernel image path to replace
        #[arg(short, long)]
        kernel: Option<PathBuf>,

        /// LKM module path to replace
        #[arg(short, long, requires("init"))]
        module: Option<PathBuf>,

        /// init to be replaced, if use LKM, this must be specified
        #[arg(short, long, requires("module"))]
        init: Option<PathBuf>,

        /// will use another slot when boot image is not specified
        #[arg(short = 'u', long, default_value = "false")]
        ota: bool,

        /// Flash it to boot partition after patch
        #[arg(short, long, default_value = "false")]
        flash: bool,

        /// output path, if not specified, will use current directory
        #[arg(short, long, default_value = None)]
        out: Option<PathBuf>,

        /// magiskboot path, if not specified, will use builtin one
        #[arg(long, default_value = None)]
        magiskboot: Option<PathBuf>,
    },
    /// For developers
    Debug {
        #[command(subcommand)]
        command: Debug,
    },
}
#[derive(clap::Subcommand, Debug)]
enum Debug {
    /// Set the manager app, kernel CONFIG_KSU_DEBUG should be enabled.
    SetManager {
        /// manager package name
        #[arg(default_value_t = String::from("me.weishu.kernelsu"))]
        apk: String,
    },

    /// Get apk size and hash
    GetSign {
        /// apk path
        apk: String,
    },

    /// Root Shell
    Su,

    /// Get kernel version
    Version,

    Mount,

    /// Copy sparse file
    Xcp {
        /// source file
        src: String,
        /// destination file
        dst: String,
    },

    /// Punch hole file
    PunchHole {
        /// file path
        file: String,
    },

    /// For testing
    Test,
}

#[derive(clap::Subcommand, Debug)]
enum Sepolicy {
    /// Patch sepolicy
    Patch {
        /// sepolicy statements
        sepolicy: String,
    },

    /// Apply sepolicy from file
    Apply {
        /// sepolicy file path
        file: String,
    },

    /// Check if sepolicy statement is supported/valid
    Check {
        /// sepolicy statements
        sepolicy: String,
    },
}

#[derive(clap::Subcommand, Debug)]
enum Module {
    /// Install module <ZIP>
    Install {
        /// module zip file path
        zip: String,
    },

    /// Uninstall module <id>
    Uninstall {
        /// module id
        id: String,
    },

    /// enable module <id>
    Enable {
        /// module id
        id: String,
    },

    /// disable module <id>
    Disable {
        // module id
        id: String,
    },

    /// list all modules
    List,

    /// Shrink module image size
    Shrink,

    /// Serve module webroot
    Serve {
        /// module id
        id: String,

        /// port
        #[arg(default_value = "8080")]
        port: u16,
    },
}

#[derive(clap::Subcommand, Debug)]
enum Profile {
    /// get root profile's selinux policy of <package-name>
    GetSepolicy {
        /// package name
        package: String,
    },

    /// set root profile's selinux policy of <package-name> to <profile>
    SetSepolicy {
        /// package name
        package: String,
        /// policy statements
        policy: String,
    },

    /// get template of <id>
    GetTemplate {
        /// template id
        id: String,
    },

    /// set template of <id> to <template string>
    SetTemplate {
        /// template id
        id: String,
        /// template string
        template: String,
    },

    /// delete template of <id>
    DeleteTemplate {
        /// template id
        id: String,
    },

    /// list all templates
    ListTemplates,
}

pub fn run() -> Result<()> {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace) // limit log level
            .with_tag("KernelSU"), // logs will show under mytag tag
    );

    #[cfg(not(target_os = "android"))]
    env_logger::init();

    // the kernel executes su with argv[0] = "su" and replace it with us
    let arg0 = std::env::args().next().unwrap_or_default();
    if arg0 == "su" || arg0 == "/system/bin/su" {
        return crate::ksu::root_shell();
    }

    let cli = Args::parse();

    log::info!("command: {:?}", cli.command);

    let result = match cli.command {
        Commands::PostFsData => event::on_post_data_fs(),
        Commands::BootCompleted => event::on_boot_completed(),

        Commands::Module { command } => {
            #[cfg(any(target_os = "linux", target_os = "android"))]
            {
                utils::switch_mnt_ns(1)?;
                utils::unshare_mnt_ns()?;
            }
            match command {
                Module::Install { zip } => module::install_module(&zip),
                Module::Uninstall { id } => module::uninstall_module(&id),
                Module::Enable { id } => module::enable_module(&id),
                Module::Disable { id } => module::disable_module(&id),
                Module::List => module::list_modules(),
                Module::Shrink => module::shrink_ksu_images(),
                Module::Serve { id, port } => server::serve_module(&id, port),
            }
        }
        Commands::Install => event::install(),
        Commands::Sepolicy { command } => match command {
            Sepolicy::Patch { sepolicy } => crate::sepolicy::live_patch(&sepolicy),
            Sepolicy::Apply { file } => crate::sepolicy::apply_file(file),
            Sepolicy::Check { sepolicy } => crate::sepolicy::check_rule(&sepolicy),
        },
        Commands::Services => event::on_services(),
        Commands::Profile { command } => match command {
            Profile::GetSepolicy { package } => crate::profile::get_sepolicy(package),
            Profile::SetSepolicy { package, policy } => {
                crate::profile::set_sepolicy(package, policy)
            }
            Profile::GetTemplate { id } => crate::profile::get_template(id),
            Profile::SetTemplate { id, template } => crate::profile::set_template(id, template),
            Profile::DeleteTemplate { id } => crate::profile::delete_template(id),
            Profile::ListTemplates => crate::profile::list_templates(),
        },

        Commands::Debug { command } => match command {
            Debug::SetManager { apk } => debug::set_manager(&apk),
            Debug::GetSign { apk } => {
                let sign = apk_sign::get_apk_signature(&apk)?;
                println!("size: {:#x}, hash: {}", sign.0, sign.1);
                Ok(())
            }
            Debug::Version => {
                println!("Kernel Version: {}", crate::ksu::get_version());
                Ok(())
            }
            Debug::Su => crate::ksu::grant_root(),
            Debug::Mount => event::mount_systemlessly(defs::MODULE_DIR),
            Debug::Xcp { src, dst } => {
                utils::copy_sparse_file(src, dst)?;
                Ok(())
            }
            Debug::PunchHole { file } => utils::punch_hole(file),
            Debug::Test => todo!(),
        },

        Commands::BootPatch {
            boot,
            init,
            kernel,
            module,
            ota,
            flash,
            out,
            magiskboot,
        } => crate::boot_patch::patch(boot, kernel, module, init, ota, flash, out, magiskboot),
    };

    if let Err(e) = &result {
        log::error!("Error: {:?}", e);
    }
    result
}

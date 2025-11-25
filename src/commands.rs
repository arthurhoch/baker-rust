use crate::cli::{parse, Command};
use crate::logger::Logger;
use crate::recipe::{decrypt_secrets, encrypt_recipe_file, parse as parse_recipe};
use crate::repository::{download, ListRecipes, Repository};
use crate::secret::{Crypto, SecretKey};
use crate::settings::Settings;
use crate::template;
use std::error::Error;

pub fn execute_command_line(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut args = args;
    if !args.is_empty() {
        args.remove(0); // binary name
    }
    let logger = Logger::new(false);
    let options = parse(&args, &logger)?;
    let settings = Settings::load(options.verbose)?;
    let logger = Logger::new(settings.debug);

    match options.command {
        Command::Help => {
            crate::cli::print_help();
        }
        Command::Version => {
            println!("baker-rust {}", crate::settings::VERSION);
        }
        Command::Configs { all } => {
            for (key, value) in settings.values(!all) {
                logger.log(&format!("{}={}", key, value));
            }
        }
        Command::Encrypt { plantexts, file } => {
            let key = SecretKey::read(&settings)?;
            let crypto = Crypto::new(key);
            if let Some(path) = file {
                encrypt_recipe_file(&path, &settings, &crypto)?;
                logger.log("Secrets encrypted in recipe file");
            } else if !plantexts.is_empty() {
                for text in plantexts {
                    let cipher = crypto.encrypt(&text)?;
                    logger.log(&format!("{} {}", text, cipher));
                }
            } else {
                return Err("encrypt expected at least one argument".into());
            }
        }
        Command::GenKey { keypass } => {
            let stored = SecretKey::generate(&keypass, &settings)?;
            logger.log(&format!(
                "Generated secret key '{}' and saved at '{}'",
                stored,
                settings.storage_key_path.display()
            ));
        }
        Command::Pull { name, force } => {
            let mut repo = Repository::new(&name, &settings)?;
            repo.pull(force, &logger)?;
        }
        Command::Recipes { all } => {
            ListRecipes::list(all, &settings, &logger)?;
        }
        Command::Rm { recipe_id } => {
            Repository::remove(&recipe_id, &settings, &logger)?;
        }
        Command::Run { name, path, force } => {
            logger.log("Baker start <:::> \n");
            let recipe_path = if let Some(name) = name {
                let mut repo = Repository::new(&name, &settings)?;
                repo.pull(force, &logger)?;
                repo.local_path
                    .ok_or("Repository pull did not set local path")?
            } else if let Some(path) = path {
                path
            } else {
                return Err("run expects a recipe name or --path".into());
            };

            let mut recipe = parse_recipe(&recipe_path, &settings, None)?;

            if recipe.instructions.iter().any(|i| !i.secrets.is_empty()) {
                let key = SecretKey::read(&settings)?;
                let crypto = Crypto::new(key);
                decrypt_secrets(&mut recipe.instructions, &crypto, recipe.case_sensitive)?;
            }

            for instruction in recipe.instructions.iter_mut() {
                if instruction.is_remote {
                    let downloaded = download(
                        &instruction.template.template,
                        None,
                        force,
                        &settings,
                        &logger,
                    )?;
                    instruction.template.template = downloaded.to_string_lossy().to_string();
                }
            }

            template::replace(&recipe.instructions, &settings, &logger)?;
            logger.log("\nAll done with success! \\ o /");
        }
    }

    Ok(())
}

use serde::Deserialize;
use once_cell::sync::OnceCell;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub adapter: Option<String>,
    pub target: String,
}

static SETTINGS: OnceCell<Settings> = OnceCell::new();

pub fn load_settings(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path).unwrap();
    let settings: Settings = serde_yaml::from_reader(file)?;
    SETTINGS.set(settings.clone()).expect("Tried to load settings file more than once");
    println!("Loaded settings: {:#?}", settings);
    Ok(())
}

pub fn get() -> &'static Settings {
    SETTINGS.get().expect("Tried to access settings before loading them")
}

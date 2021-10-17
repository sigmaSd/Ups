use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, io::ErrorKind, path::PathBuf, process::Command};

use crate::colors::Color;

mod colors;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const NONE: &str = "NONE";

fn main() -> Result<()> {
    let mut ups = Ups::default();
    let ups: &mut dyn Actions = &mut ups;
    ups.load()?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(|a| a.as_str())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => {
            ups.update_latest_value()?;
            ups.print();
        }
        ["insert", name, script_path] => ups.insert(name.to_string(), script_path)?,
        ["snapshot", name] => ups.snapshot(name)?,
        ["get", name] => println!("{}", ups.latest_value(name)?),
        _ => unimplemented!(),
    }
    Ok(())
}

trait Actions: Drop {
    fn insert(&mut self, name: String, script_path: &str) -> Result<()>;
    fn snapshot(&mut self, name: &str) -> Result<()>;
    fn latest_value(&self, name: &str) -> Result<String>;
    fn update_latest_value(&mut self) -> Result<()>;
    fn print(&self);
    fn load(&mut self) -> Result<()>;
    fn save(&self) -> Result<()>;
}

#[derive(Debug)]
struct App {
    script_path: PathBuf,
    latest_value: Option<String>,
    snapshot_value: Option<String>,
}
#[derive(Default)]
struct Ups {
    apps: HashMap<String, App>,
}
impl Actions for Ups {
    fn insert(&mut self, name: String, script_path: &str) -> Result<()> {
        self.apps.insert(
            name,
            App {
                script_path: Path::new(script_path).canonicalize()?,
                latest_value: None,
                snapshot_value: None,
            },
        );
        Ok(())
    }

    fn snapshot(&mut self, name: &str) -> Result<()> {
        let latest_value = self.latest_value(name)?;
        let app = self.apps.get_mut(name).expect("Already checked");
        app.latest_value = Some(latest_value.clone());
        app.snapshot_value = Some(latest_value);
        Ok(())
    }

    fn print(&self) {
        println!();
        println!(
            "{}\t{}\t{}\t{}",
            "App".light_blue(),
            "SnapshotValue".light_blue(),
            "LatestValue".light_blue(),
            "ScriptPath".light_blue()
        );

        for (name, app) in &self.apps {
            let diff_color: fn(&str) -> String = if app.snapshot_value == app.latest_value {
                Color::green
            } else {
                Color::red
            };
            println!(
                "{}\t{}\t{}\t{}",
                name.yellow(),
                diff_color(app.snapshot_value.as_ref().unwrap_or(&NONE.to_string())),
                diff_color(app.latest_value.as_ref().unwrap_or(&NONE.to_string())),
                app.script_path.display().to_string().rgb(100, 80, 250)
            );
        }
    }

    fn save(&self) -> Result<()> {
        let mut data = std::fs::File::create(data_path()?)?;

        for (name, app) in &self.apps {
            writeln!(
                data,
                "{}\t{}\t{}\t{}\t",
                name,
                app.snapshot_value.as_ref().unwrap_or(&NONE.to_string()),
                app.latest_value.as_ref().unwrap_or(&NONE.to_string()),
                app.script_path.display()
            )?;
        }
        Ok(())
    }

    fn load(&mut self) -> Result<()>
    where
        Self: Sized,
    {
        let data_path = data_path()?;
        if !data_path.exists() {
            return Ok(());
        }

        let data = std::fs::read_to_string(data_path)?;
        const PARSE_ERROR: &str = "Error while parsing data file";

        let mut apps = HashMap::new();
        for line in data.lines() {
            let mut line = line.split_whitespace();
            let name = line.next().ok_or(PARSE_ERROR)?;
            let snapshot_value = line.next().ok_or(PARSE_ERROR)?;
            let latest_value = line.next().ok_or(PARSE_ERROR)?;
            let script_path = line.next().ok_or(PARSE_ERROR)?;
            apps.insert(
                name.into(),
                App {
                    script_path: script_path.into(),
                    latest_value: if latest_value != NONE {
                        Some(latest_value.into())
                    } else {
                        None
                    },
                    snapshot_value: if snapshot_value != NONE {
                        Some(snapshot_value.into())
                    } else {
                        None
                    },
                },
            );
        }
        self.apps = apps;
        Ok(())
    }

    fn latest_value(&self, name: &str) -> Result<String> {
        let app = self
            .apps
            .get(name)
            .ok_or(format!("App `{}` is not registered.", name))?;
        print!(
            "{}",
            format!("Fetching latest value of `{}` app...", name).yellow()
        );
        std::io::stdout().flush()?;
        let output = Command::new(&app.script_path).output()?;
        if output.status.success() {
            println!("{}", "Ok".green());
        } else {
            return Err(format!("Failed:\n{}", String::from_utf8(output.stderr)?).into());
        }
        let value = String::from_utf8(output.stdout)?;
        let value = value.trim();
        if value.is_empty() {
            Ok(NONE.to_owned())
        } else {
            Ok(value.to_owned())
        }
    }

    fn update_latest_value(&mut self) -> Result<()> {
        let apps: Vec<_> = self.apps.iter().map(|(name, _)| name.clone()).collect();
        for name in apps {
            let latest_value = self.latest_value(&name)?;
            self.apps
                .get_mut(&name)
                .expect("Already checked")
                .latest_value = Some(latest_value);
        }
        Ok(())
    }
}

impl Drop for Ups {
    fn drop(&mut self) {
        let _ = self.save();
    }
}

fn data_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or("Can not find xdg_data_dir")?
        .join("ups");
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        if e.kind() != ErrorKind::AlreadyExists {
            return Err(e.into());
        }
    }
    Ok(data_dir.join("data"))
}

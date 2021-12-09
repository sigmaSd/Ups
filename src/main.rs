use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, io::ErrorKind, path::PathBuf, process::Command};

use scolor::ColorExt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const NONE: &str = "NONE";

fn main() -> Result<()> {
    let mut ups = Ups::default();
    let ups: &mut dyn Actions = &mut ups;
    ups.load()?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
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
        _ => println!("{}", usage()),
    }
    Ok(())
}

#[allow(drop_bounds)]
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
    latest_value: String,
    snapshot_value: String,
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
                latest_value: NONE.to_owned(),
                snapshot_value: NONE.to_owned(),
            },
        );
        Ok(())
    }

    fn snapshot(&mut self, name: &str) -> Result<()> {
        let latest_value = self.latest_value(name)?;
        let app = self.apps.get_mut(name).expect("Already checked");
        app.latest_value = latest_value.clone();
        app.snapshot_value = latest_value;
        Ok(())
    }

    fn print(&self) {
        use term_table::row::Row;
        use term_table::table_cell::TableCell;
        use term_table::{Table, TableStyle};

        let mut table = Table::new();
        table.style = TableStyle::rounded();

        table.add_row(Row::new(vec![
            TableCell::new("App".light_blue().underline()),
            TableCell::new("SnapshotValue".light_blue().underline()),
            TableCell::new("LatestValue".light_blue().underline()),
            TableCell::new("ScriptPath".light_blue().underline()),
        ]));

        for (name, app) in &self.apps {
            let diff_color = if app.snapshot_value == app.latest_value {
                ColorExt::green
            } else {
                ColorExt::red
            };
            table.add_row(Row::new(vec![
                TableCell::new(name.yellow().bold()),
                TableCell::new(diff_color(&app.snapshot_value)),
                TableCell::new(diff_color(&app.latest_value)),
                TableCell::new(app.script_path.display().rgb(100, 80, 250).italic()),
            ]));
        }
        println!("\n{}", table.render());
    }

    fn save(&self) -> Result<()> {
        let mut data = std::fs::File::create(data_path()?)?;

        for (name, app) in &self.apps {
            writeln!(
                data,
                "{}\t{}\t{}\t{}\t",
                name,
                app.snapshot_value,
                app.latest_value,
                app.script_path.display()
            )?;
        }
        Ok(())
    }

    fn load(&mut self) -> Result<()>
    where
        Self: Sized,
    {
        const PARSE_ERROR: &str = "Error while parsing data file";
        let data_path = data_path()?;
        if !data_path.exists() {
            return Ok(());
        }

        let data = std::fs::read_to_string(data_path)?;

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
                    latest_value: latest_value.into(),
                    snapshot_value: snapshot_value.into(),
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
        let value = String::from_utf8(output.stdout)?;
        let value = value.trim();

        if output.status.success() && !value.is_empty() {
            println!("{}", "Ok".green().bold());
            Ok(value.to_owned())
        } else {
            println!("{}", "Failed".red().bold());
            return Ok(NONE.to_owned());
        }
    }

    fn update_latest_value(&mut self) -> Result<()> {
        let apps: Vec<_> = self.apps.iter().map(|(name, _)| name.clone()).collect();
        for name in apps {
            let latest_value = self.latest_value(&name)?;
            self.apps
                .get_mut(&name)
                .expect("Already checked")
                .latest_value = latest_value;
        }
        Ok(())
    }
}

impl Drop for Ups {
    fn drop(&mut self) {
        if let Err(e) = self.save() {
            eprintln!("Failed to save data:\n{}", e);
        }
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

const fn usage() -> &'static str {
    "Ups: Check for app's updates

    - ups # Check for updates
    - ups insert [app] [check_update_script_path] # Insert an app into ups
    - ups snapshot [app] # Snapshot latest version"
}

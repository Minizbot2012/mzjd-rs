use mzjd::{Op, Operation};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    env::args,
    fs::{self, File},
    path::Path,
};

#[derive(Deserialize)]
pub struct UpdateService {
    pub marker: String,
    pub index: Vec<String>,
}

pub fn get_file(path: &str, defaul: Value) -> Value {
    if fs::metadata(path).is_ok() {
        serde_json::from_reader(File::open(path).unwrap()).unwrap()
    } else {
        defaul
    }
}

fn main() {
    let mut arg = args().skip(1);
    let gam = arg.next().unwrap();
    let game_dir = Path::new(&gam);
    let mut base: Value = get_file(
        "database.base.json",
        json!(
            {
                "DBVer": 0,
                "UpdateLocation": "https://raw.githubusercontent.com/minis-patchers/SynDelta/main/SynWeaponKeywords/index.json",
                "DoUpdates": true,
                "Marker": "main-mzjd-v1"
            }
        ),
    );
    serde_json::to_writer_pretty(File::create("database.base.json").unwrap(), &base).unwrap();
    let update_server = base.get("UpdateLocation").unwrap().as_str().unwrap();
    let index: UpdateService = reqwest::blocking::get(update_server)
        .unwrap()
        .json()
        .unwrap();
    if base.get("Marker").unwrap().as_str().unwrap() != index.marker {
        println!("Resetting due to marker change");
        base = json!(
            {
                "DBVer": 0,
                "UpdateLocation": update_server,
                "DoUpdates": true,
                "Marker": index.marker,
            }
        )
    }
    if base.get("DoUpdates").unwrap().as_bool().unwrap() {
        for i in base.get("DBVer").unwrap().as_u64().unwrap() as usize..index.index.len() {
            let patch: Vec<Operation> = reqwest::blocking::get(index.index.get(i).unwrap())
                .unwrap()
                .json()
                .unwrap();
            println!("Applying patch for DB V{i}");
            for p in patch {
                p.apply(&mut base);
            }
            serde_json::to_writer_pretty(File::create("database.base.json").unwrap(), &base)
                .unwrap();
        }
    }
    let mut game = base.clone();
    for file in fs::read_dir(game_dir).unwrap() {
        let fi = file.unwrap();
        let filn = fi.file_name().into_string().unwrap();
        if filn.ends_with("_SWK.json") {
            let patch: Vec<Operation> =
                serde_json::from_reader(File::open(fi.path()).unwrap()).unwrap();
            println!("Applying patch for {filn}");
            for p in patch {
                p.apply(&mut game)
            }
        }
    }
    serde_json::to_writer_pretty(File::create("database.json").unwrap(), &game).unwrap();
}

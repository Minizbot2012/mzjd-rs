use mzjd::{apply_diff, diff_tree, Operation};
use serde_json::Value;
use std::{env::args, fs::File};

fn main() {
    let mut arg = args().skip(1);
    match arg.next().unwrap().as_str() {
        "diff" => {
            let left: Value =
                serde_json::from_reader(File::open(arg.next().unwrap()).unwrap()).unwrap();
            let right: Value =
                serde_json::from_reader(File::open(arg.next().unwrap()).unwrap()).unwrap();
            let mut path = Vec::new();
            let diff = diff_tree(&left, &right, &mut path);
            let mut out = File::create(arg.next().unwrap()).unwrap();
            serde_json::to_writer_pretty(&mut out, &diff).unwrap();
        }
        "patch" => {
            let ops: Vec<Operation> =
                serde_json::from_reader(File::open(arg.next().unwrap()).unwrap()).unwrap();
            let left: Value =
                serde_json::from_reader(File::open(arg.next().unwrap()).unwrap()).unwrap();
            let output = apply_diff(ops, left);
            let mut out = File::create(arg.next().unwrap()).unwrap();
            serde_json::to_writer_pretty(&mut out, &output).unwrap();
        }
        _ => {}
    }
}

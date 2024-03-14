use serde::{Deserialize, Serialize};
use serde_json::Value;

pub trait Op {
    fn apply(self, input: &mut Value);
}
#[derive(Serialize, Deserialize)]
pub struct AddOp {
    pub path: String,
    pub value: Value,
}

impl Op for AddOp {
    fn apply(self, input: &mut Value) {
        let pv = self.path.clone();
        let point = pv.rfind("/").unwrap();
        let (split, vn) = pv.split_at(point);
        let ptr = input.pointer_mut(&split).unwrap();
        match ptr.pointer(&format!("/{vn}")) {
            Some(_) => match ptr.pointer_mut(&format!("/{vn}")).unwrap() {
                Value::Array(v) => {
                    if !v.contains(&self.value) {
                        v.push(self.value)
                    }
                }
                _ => {}
            },
            None => match ptr {
                Value::Object(obj) => {
                    obj.insert(vn.to_string(), self.value.clone());
                }
                _ => {}
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RemoveOp {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

impl Op for RemoveOp {
    fn apply(self, input: &mut Value) {
        let pv = self.path.clone();
        let point = pv.rfind("/").unwrap();
        let (split, vn) = pv.split_at(point);
        let ptr = input.pointer_mut(&split).unwrap();
        match ptr.pointer(&format!("/{vn}")) {
            Some(x) => match x {
                Value::Array(_) => {
                    let v = ptr
                        .pointer_mut(&format!("/{vn}"))
                        .unwrap()
                        .as_array_mut()
                        .unwrap();
                    let val = self.value.unwrap();
                    if v.contains(&val) {
                        match v.iter().position(|v| val == *v) {
                            Some(pos) => {
                                v.remove(pos);
                            }
                            None => {}
                        }
                    }
                }
                _ => {
                    ptr.as_object_mut().unwrap().remove(vn);
                }
            },
            None => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SetOp {
    pub path: String,
    pub value: Value,
}

impl Op for SetOp {
    fn apply(self, input: &mut Value) {
        *input.pointer_mut(&self.path).unwrap() = self.value.clone();
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum Operation {
    Add(AddOp),
    Remove(RemoveOp),
    Set(SetOp),
}

impl Op for Operation {
    fn apply(self, input: &mut Value) {
        match self {
            Operation::Add(o) => o.apply(input),
            Operation::Remove(o) => o.apply(input),
            Operation::Set(o) => o.apply(input),
        }
    }
}

pub fn diff_tree(left: &Value, right: &Value) -> Vec<Operation> {
    diff_tree_internal(left, right, &mut Vec::new())
}

fn diff_tree_internal(left: &Value, right: &Value, path: &mut Vec<String>) -> Vec<Operation> {
    let mut op = Vec::new();
    let mut pointer = path.join("/");
    if path.len() == 0 {
        pointer = "".into()
    } else {
        pointer.insert(0, '/');
    }
    match left.clone() {
        Value::Array(l) => {
            if right.is_array() {
                let mut idx = 0;
                let ra = right.as_array().unwrap();
                let la = l;
                for item in ra.iter() {
                    if item.is_array() || item.is_object() {
                        path.push(format!("{idx}"));
                        op.append(&mut diff_tree_internal(left.get(idx).unwrap(), item, path));
                        path.pop();
                    } else if !la.contains(item) {
                        op.push(Operation::Add(AddOp {
                            path: format!("{pointer}"),
                            value: item.clone(),
                        }))
                    }
                    idx += 1;
                }
                idx = 0;
                for item in la.iter() {
                    if item.is_array() || item.is_object() {
                        path.push(format!("{idx}"));
                        op.append(&mut diff_tree_internal(item, ra.get(idx).unwrap(), path));
                        path.pop();
                    } else if !ra.contains(item) {
                        op.push(Operation::Remove(RemoveOp {
                            path: format!("{pointer}"),
                            value: Some(item.clone()),
                        }))
                    }
                    idx += 1;
                }
            } else {
                op.push(Operation::Remove(RemoveOp {
                    path: format!("{pointer}"),
                    value: Some(left.clone()),
                }));
                op.push(Operation::Add(AddOp {
                    path: format!("{pointer}"),
                    value: right.clone(),
                }));
            }
        }
        Value::Bool(l) => {
            if right.is_boolean() {
                if l != right.as_bool().unwrap() {
                    op.push(Operation::Set(SetOp {
                        path: format!("{pointer}"),
                        value: right.clone(),
                    }))
                }
            } else {
                op.push(Operation::Remove(RemoveOp {
                    path: format!("{pointer}"),
                    value: None,
                }));
                op.push(Operation::Add(AddOp {
                    path: format!("{pointer}"),
                    value: right.clone(),
                }));
            }
        }
        Value::Null => {}
        Value::Number(l) => {
            if right.is_number() {
                if l != *right.as_number().unwrap() {
                    op.push(Operation::Set(SetOp {
                        path: format!("{pointer}"),
                        value: right.clone(),
                    }))
                }
            } else {
                op.push(Operation::Remove(RemoveOp {
                    path: format!("{pointer}"),
                    value: None,
                }));
                op.push(Operation::Add(AddOp {
                    path: format!("{pointer}"),
                    value: right.clone(),
                }));
            }
        }
        Value::Object(l) => {
            if right.is_object() {
                for (k, v) in l.iter() {
                    if right.as_object().unwrap().contains_key(k) {
                        path.push(k.clone());
                        op.append(&mut diff_tree_internal(v, right.get(k).unwrap(), path));
                        path.pop();
                    } else {
                        op.push(Operation::Remove(RemoveOp {
                            path: format!("{pointer}/{k}"),
                            value: None,
                        }));
                    }
                }
                for (k, v) in right.as_object().unwrap().iter() {
                    if l.contains_key(k) {
                        path.push(k.clone());
                        op.append(&mut diff_tree_internal(v, right.get(k).unwrap(), path));
                        path.pop();
                    } else {
                        op.push(Operation::Add(AddOp {
                            path: format!("{pointer}/{k}"),
                            value: right.get(k).unwrap().clone(),
                        }));
                    }
                }
            } else {
                op.push(Operation::Remove(RemoveOp {
                    path: format!("{pointer}"),
                    value: None,
                }));
                op.push(Operation::Add(AddOp {
                    path: format!("{pointer}"),
                    value: right.clone(),
                }));
            }
        }
        Value::String(l) => {
            if right.is_string() {
                if l != *right.as_str().unwrap() {
                    op.push(Operation::Set(SetOp {
                        path: format!("{pointer}"),
                        value: right.clone(),
                    }))
                }
            } else {
                op.push(Operation::Remove(RemoveOp {
                    path: format!("{pointer}"),
                    value: None,
                }));
                op.push(Operation::Add(AddOp {
                    path: format!("{pointer}"),
                    value: right.clone(),
                }));
            }
        }
    }
    op
}

pub fn apply_diff(operations: Vec<Operation>, input: Value) -> Value {
    let mut out = input.clone();
    for op in operations {
        op.apply(&mut out);
    }
    out
}

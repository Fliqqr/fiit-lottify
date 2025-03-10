use std::{fs::File, io::Write};

use serde_json::{self, Value};

use crate::draw::{round_to, MeshShape};

pub fn base_lottie(framerate: u64, frames: u64) -> Value {
    serde_json::from_str::<Value>(&format!(
        r#"{{
            "v": "5.7.5",
            "fr": {},
            "ip": 0,
            "op": {},
            "w": 1200,
            "h": 900,
            "nm": "Comp 1",
            "ddd": 0,
            "metadata": {{}},
            "assets": [],
            "layers": [],
            "markers": []
        }}"#,
        framerate, frames
    ))
    .unwrap()
}

pub fn add_layer(json: &mut Value, layer: Value) {
    json["layers"].as_array_mut().unwrap().push(layer);
}

pub fn new_layer(name: &str, ip: u64, op: u64) -> Value {
    serde_json::from_str::<Value>(&format!(
        r#"{{
            "ddd": 0,
            "ind": 0,
            "ty": 4,
            "nm": "{}",
            "sr": 1,
            "ao": 0,
            "shapes": [],
            "ip": {},
            "st": {},
            "op": {},
            "bm": 0,
            "ks": {{
                "p": {{
                    "a": 0,
                    "k": [
                        600,
                        450
                    ],
                    "ix": 2
                }},
                "a": {{
                    "a": 0,
                    "k": [
                        0,
                        0
                    ],
                    "ix": 2
                }},
                "s": {{
                    "a": 0,
                    "k": [
                        100,
                        100
                    ],
                    "ix": 2
                }},
                "r": {{
                    "a": 0,
                    "k": 0,
                    "ix": 2
                }},
                "o": {{
                    "a": 1,
                    "k": [
                        {{
                            "t": 0,
                            "s": [
                                100
                            ],
                            "h": 1
                        }},
                        {{
                            "t": 50,
                            "s": [
                                0
                            ],
                            "h": 1
                        }}
                    ],
                    "ix": 2
                }},
                "sk": {{
                    "a": 0,
                    "k": 0,
                    "ix": 2
                }},
                "sa": {{
                    "a": 0,
                    "k": 0,
                    "ix": 2
                }}
            }}
        }}"#,
        // ip + 1,
        name,
        ip,
        ip,
        op,
    ))
    .unwrap()
}

struct Group {
    name: String,
    items: Vec<Value>,
}

impl Group {
    const BASE: &'static str = r#"{
        "ty": "gr",
        "nm": "",
        "it": []
    }"#;

    pub fn new(name: String) -> Self {
        Self {
            name,
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, item: Value) {
        self.items.push(item);
    }

    pub fn to_value(&self) -> Value {
        let mut base = serde_json::from_str::<Value>(Self::BASE).unwrap();
        base["nm"] = serde_json::to_value(&self.name).unwrap();
        base["it"]
            .as_array_mut()
            .unwrap()
            .extend(self.items.clone());
        base
    }
}

struct Stroke {
    items: Vec<(f64, f64)>,
}

impl Stroke {
    const BASE: &'static str = r#"{
        "ty": "sh",
        "d": 1,
        "ks": {
            "a": 0,
            "k": {
                "c": true,
                "v": [],
                "i": [],
                "o": []
            }
        }
    }"#;

    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, item: (f64, f64)) {
        self.items.push(item)
    }

    pub fn to_value(&self) -> Value {
        let mut base = serde_json::from_str::<Value>(Self::BASE).unwrap();
        base["ks"]["k"]["v"] = serde_json::to_value(self.items.clone()).unwrap();
        base
    }
}

struct Fill {}

impl Fill {
    const BASE: &'static str = r#"{
        "ty": "fl",
        "c": {
            "a": 0,
            "k": [],
            "ix": 2
        },
        "o": {
            "a": 0,
            "k": 100,
            "ix": 2
        },
        "r": 1,
        "bm": 0
    }"#;

    pub fn color(rgb: [f32; 3]) -> Value {
        let mut base = serde_json::from_str::<Value>(Self::BASE).unwrap();
        base["c"]["k"] = serde_json::to_value(rgb).unwrap();
        base
    }
}

#[derive(Debug)]
pub struct Lottie {
    layers: Vec<Value>,
    framerate: u64,
}

impl Lottie {
    pub fn new(framerate: u64) -> Self {
        Lottie {
            layers: Vec::new(),
            framerate,
        }
    }

    pub fn serialize(&self) -> String {
        let mut base = base_lottie(self.framerate, self.layers.len() as u64);

        for layer in &self.layers {
            add_layer(&mut base, layer.clone());
        }

        base.to_string()
    }

    pub fn save_as(&self, file_name: &str) {
        let mut handle = File::create(file_name).unwrap();

        // println!("Saving: {:?}", self);

        handle.write_all(self.serialize().as_bytes()).unwrap();
    }

    pub fn add_layer(&mut self, mesh_shapes: Vec<MeshShape>, name: &str, ip: u64, op: u64) {
        let mut layer = new_layer(name, ip, op);

        for ms in &mesh_shapes {
            let mut shape = Group::new("group".into());

            let mut stroke = Stroke::new();

            // Assuming every sub-shape ends with ClosePath
            for p in &ms.shape.paths {
                if let Some(point) = p.end_point() {
                    stroke.add((round_to(point.x, 3), round_to(point.y, 3)));
                } else {
                    // end subgroup
                    shape.add(stroke.to_value());
                    stroke = Stroke::new();
                }
            }

            let color = ms.color.to_linear();
            shape.add(Fill::color([color.red, color.green, color.blue]));

            layer["shapes"]
                .as_array_mut()
                .unwrap()
                .push(shape.to_value());
        }

        self.layers.push(layer);
    }
}

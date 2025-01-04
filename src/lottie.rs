use std::{fs::File, io::Write};

use serde_json::{self, Value};

pub fn base_lottie() -> Value {
    serde_json::from_str::<Value>(
        r#"{
            "v": "5.7.5",
            "fr": 100,
            "ip": 0,
            "op": 300,
            "w": 1200,
            "h": 900,
            "nm": "Comp 1",
            "ddd": 0,
            "metadata": {},
            "assets": [],
            "layers": [],
            "markers": []
        }"#,
    )
    .unwrap()
}

pub fn add_layer(json: &mut Value, layer: Value) {
    json["layers"].as_array_mut().unwrap().push(layer);
}

pub fn new_layer() -> Value {
    serde_json::from_str::<Value>(
        r#"{
            "ddd": 0,
            "ind": 1,
            "ty": 4,
            "nm": "Shape Layer 1",
            "sr": 1,
            "ao": 0,
            "shapes": [],
            "ip": 0,
            "op": 301,
            "st": 0,
            "bm": 0
        }"#,
    )
    .unwrap()
}

pub fn add_shape(layer: &mut Value, indices: &Vec<(f64, f64)>, color: &(f64, f64, f64)) {
    let mut group = serde_json::from_str::<Value>(
        r#"{
            "ty": "gr",
            "nm": "Shape Layer 1",
            "it": [
                {
                    "ty": "sh",
                    "d": 1,
                    "ks": {
                        "a": 0,
                        "k": {
                            "c": true,
                            "v": []
                        }
                    }
                },
                {
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
                },
                {
                    "ty": "st",
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
                    "w": {
                        "a": 0,
                        "k": 2,
                        "ix": 2
                    },
                    "lc": 1,
                    "lj": 1,
                    "ml": 4
                },
            ]
        }
    "#,
    )
    .unwrap();

    group["it"][0]["ks"]["k"]["v"] = serde_json::to_value(indices).unwrap();

    group["it"][1]["c"]["k"] = serde_json::to_value(color).unwrap();
    group["it"][2]["c"]["k"] = serde_json::to_value(color).unwrap();

    layer["shapes"].as_array_mut().unwrap().push(group);
}

#[derive(Debug)]
pub struct Shape {
    indices: Vec<(f64, f64)>,
    color: (f64, f64, f64),
}

impl Shape {
    pub fn scale(mut self, factor: f64) -> Self {
        for (x, y) in &mut self.indices.iter_mut() {
            *x *= factor;
            *y *= factor;
        }
        self
    }
}

#[derive(Debug)]
pub struct Lottie {
    shapes: Vec<Shape>,
}

impl Lottie {
    pub fn new() -> Self {
        Lottie { shapes: Vec::new() }
    }

    pub fn add_shape(&mut self, indices: Vec<(f64, f64)>, color: (f64, f64, f64)) {
        self.shapes.push(Shape { indices, color }.scale(100.0));
    }

    pub fn serialize(&self) -> String {
        let mut base = base_lottie();
        let mut layer = new_layer();

        for shape in &self.shapes {
            add_shape(&mut layer, &shape.indices, &shape.color);
        }

        add_layer(&mut base, layer);

        base.to_string()
    }

    pub fn save_as(&self, file_name: &str) {
        let mut handle = File::create(file_name).unwrap();

        println!("Saving: {:?}", self);

        handle.write_all(self.serialize().as_bytes()).unwrap();
    }
}

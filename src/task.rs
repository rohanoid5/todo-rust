pub struct Task {
    pub _id: i32,
    pub name: String,
    pub checked: bool,
}

impl Task {
    pub fn new(_id: i32, name: String, checked: bool) -> Self {
        Self { _id, name, checked }
    }
}

use std::fs::File;
use std::io::{self, Read, Write};

use crate::model::Task;
// ชื่อไฟล์ JSON ที่เก็บข้อมูล โดยอ้างอิงจากไดเรกทอรีที่ใช้รันโปรแกรม
const FILE_PATH: &str = "todos.json";

// serialize รายการเป็น JSON ที่มีการจัดรูปแบบอ่านง่าย แล้วเขียนทับไฟล์ข้อมูล
pub(crate) fn save_tasks(tasks: &[Task]) -> io::Result<()> {
    // ใช้ to_string_pretty เพื่อให้ไฟล์ JSON อ่านง่ายสำหรับผู้ใช้ และ map_err เพื่อแปลงข้อผิดพลาดเป็น io::Error
    let json = serde_json::to_string_pretty(tasks)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    // สร้างไฟล์ใหม่ทับไฟล์เดิม หากเกิดข้อผิดพลาดจะไม่ลบไฟล์เดิม
    let mut file = File::create(FILE_PATH)?;
    file.write_all(json.as_bytes())
}

// โหลดรายการ JSON ไฟล์ที่ยังไม่มีถือเป็นรายการว่าง ส่วนข้อผิดพลาดอื่นจะถูกรายงาน
pub(crate) fn load_tasks() -> io::Result<Vec<Task>> {
    let mut file = match File::open(FILE_PATH) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // แปลง JSON เป็น Vec<Task> โดย map_err เพื่อแปลงข้อผิดพลาดเป็น io::Error
    serde_json::from_str(&contents)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

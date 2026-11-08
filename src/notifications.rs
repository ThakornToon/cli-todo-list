use chrono::{Duration, Local, NaiveDateTime};

use crate::model::Task;

// ตรวจสอบงานที่ยังไม่เสร็จ และแสดงกลุ่มงานเลยกำหนดหรือถึงกำหนดภายใน 24 ชั่วโมง
pub fn check_notifications(tasks: &[Task]) {
    let now: NaiveDateTime = Local::now().naive_local();
    let (mut overdue, mut upcoming) = notification_groups(tasks, now);

    if !overdue.is_empty() || !upcoming.is_empty() {
        println!("\n🚨 ================= แจ้งเตือนด่วน! =================");
        if !overdue.is_empty() {
            println!("⚠️ งานที่เลยกำหนดแล้ว {} งาน:", overdue.len());
            print_notification_tasks(&mut overdue);
        }
        if !upcoming.is_empty() {
            println!("⏰ งานที่จะถึงกำหนดภายใน 24 ชั่วโมง {} งาน:", upcoming.len());
            print_notification_tasks(&mut upcoming);
        }
        println!("==================================================");
    }
}

// แบ่งงานที่ยังไม่เสร็จเป็นกลุ่มเลยกำหนดและใกล้ถึงกำหนด โดยใช้เวลาปัจจุบันที่ส่งเข้ามา
fn notification_groups(tasks: &[Task], now: NaiveDateTime) -> (Vec<&Task>, Vec<&Task>) {
    let mut overdue: Vec<&Task> = Vec::new();
    let mut upcoming: Vec<&Task> = Vec::new();

    // พารามิเตอร์ tasks มีชนิด &[Task] ซึ่งเป็น slice ที่ยืมอ่านข้อมูล task จึงได้ &Task ในการเก็บกลุ่มงานที่ตรงเงื่อนไข
    for task in tasks {
        if !task.completed {
            if task.due_date < now {
                overdue.push(task);
            } else if task.due_date - now <= Duration::hours(24) {
                upcoming.push(task);
            }
        }
    }
    (overdue, upcoming)
}

// เรียงงานแจ้งเตือนตามกำหนดส่ง แล้วพิมพ์ ID ชื่องาน และวันเวลา
fn print_notification_tasks(tasks: &mut [&Task]) {
    // ใน closure นี้ task มีชนิด &&Task เพราะฟังก์ชันรับ & ไปยัง Element ซึ่งเป็น &Task
    // Rust dereference ให้อัตโนมัติ จึงเขียน task.due_date ได้โดยไม่ต้องเขียน (**task).due_date
    tasks.sort_by_key(|task| task.due_date);
    for task in tasks {
        println!(
            "  • #{} {} — {}",
            task.id,
            task.title,
            task.due_date.format("%Y-%m-%d %H:%M")
        );
    }
}

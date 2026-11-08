mod display;
mod input;
mod model;
mod notifications;
mod operations;
mod storage;

use display::display_tasks;
use input::{PromptInput, prompt_calendar_filter, prompt_date_range, prompt_line, prompt_value};
use model::{CalendarFilter, DateFilter, SortOrder, Task, task_matches_filter};
use notifications::check_notifications;
use operations::{add_task, delete_task, edit_task, normalize_task_ids, toggle_task_status};
use storage::load_tasks;

fn main() {
    // หากอ่านข้อมูลไม่ได้ให้รายงานข้อผิดพลาดและหยุดโปรแกรม
    let mut tasks: Vec<Task> = match load_tasks() {
        Ok(tasks) => tasks,
        Err(error) => {
            eprintln!("❌ โหลดรายการ Task ไม่สำเร็จ: {error}");
            return;
        }
    };
    // แจ้งเตือนงานที่เลยกำหนดหรือจะถึงกำหนดใน 24 ชั่วโมงเมื่อเปิดโปรแกรม
    check_notifications(&tasks);

    loop {
        // แสดงเมนูหลัก แล้วรับอินพุตโดยรองรับการกลับเมนูและจบอินพุตด้วย
        println!("\n=== RUST TODO LIST ===");
        println!("1. เพิ่ม Task ใหม่");
        println!("2. ดู/ค้นหา Task");
        println!("3. เปลี่ยนสถานะ Task หลายรายการ (Complete/Incomplete)");
        println!("4. ลบ Task (ระบุได้หลาย ID)");
        println!("5. แก้ไขชื่องาน/กำหนดเวลา");
        println!("6. จัดเรียง ID ใหม่");
        println!("7. ออกจากโปรแกรม");
        let choice = match prompt_line("กรุณาเลือกเมนู (1-7): ") {
            PromptInput::Value(choice) => choice,
            PromptInput::Back => continue,
            PromptInput::EndOfInput => {
                println!("\nจบการทำงาน");
                break;
            }
        };

        // ตัดช่องว่างหัวท้ายก่อนเลือกคำสั่ง และเรียกฟังก์ชันของเมนูนั้น
        match choice.trim() {
            "1" => add_task(&mut tasks),
            "2" => list_tasks(&tasks),
            "3" => toggle_task_status(&mut tasks),
            "4" => delete_task(&mut tasks),
            "5" => edit_task(&mut tasks),
            "6" => normalize_task_ids(&mut tasks),
            "7" => {
                println!("ม้าที่ว่าเร็วก็ยังแพ้ลา...เพราะลาไปก่อน ✌️");
                break;
            }
            _ => println!("❌ เมนูไม่ถูกต้อง กรุณาเลือก 1-7"),
        }
    }
}

// แสดง Task หลังให้ผู้ใช้เลือกตัวกรองวันที่/สถานะและลำดับการเรียง
fn list_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("\n📭 ยังไม่มี Task ในระบบ");
        return;
    }

    println!("กรองวันที่: 1) ปี/เดือน/วัน  2) ช่วงวันที่  3) ทุกวัน");
    // ใช้ loop เพื่อให้ผู้ใช้เลือกตัวกรองวันที่ซ้ำจนกว่าจะได้ค่าที่ถูกต้อง หรือยกเลิก
    let date_filter: DateFilter = loop {
        let Some(input) = prompt_value("เลือก (1-3): ") else {
            return;
        };
        match input.trim() {
            // เลือกตัวกรองวันที่แบบปี/เดือน/วัน
            "1" => match prompt_calendar_filter() {
                Some(filter) => break filter,
                None => return,
            },
            // เลือกตัวกรองวันที่แบบช่วงวันเริ่มต้น/สิ้นสุด
            "2" => match prompt_date_range() {
                Some(range) => {
                    break DateFilter {
                        calendar: CalendarFilter::All,
                        range: Some(range),
                    };
                }
                None => return,
            },
            // เลือกตัวกรองวันที่แบบทุกวัน
            "3" => {
                break DateFilter {
                    calendar: CalendarFilter::All,
                    range: None,
                };
            }
            _ => println!("❌ กรุณาเลือก 1, 2 หรือ 3"),
        }
    };

    let status_filter: Option<bool> = loop {
        let Some(input) = prompt_value("สถานะ: 1) ยังไม่เสร็จ  2) เสร็จแล้ว  3) ทั้งหมด: ")
        else {
            return;
        };
        match input.trim() {
            "1" => break Some(false),
            "2" => break Some(true),
            "3" => break None,
            _ => println!("❌ กรุณาเลือก 1, 2 หรือ 3"),
        }
    };
    let sort_order: SortOrder = loop {
        let Some(input) = prompt_value("เรียงตาม: 1) กำหนดเวลา  2) สถานะ  3) ชื่องาน: ")
        else {
            return;
        };
        match input.trim() {
            "1" => break SortOrder::DueDate,
            "2" => break SortOrder::Status,
            "3" => break SortOrder::Title,
            _ => println!("❌ กรุณาเลือก 1, 2 หรือ 3"),
        }
    };

    // ใช้ตัวกรองวันที่และสถานะร่วมกัน แล้วส่งสำเนารายการที่ตรงเงื่อนไขไปแสดงผล
    let filtered_tasks: Vec<Task> = tasks
        .iter()
        .filter(|task| task_matches_filter(task, date_filter))
        .filter(|task| status_filter.is_none_or(|completed| task.completed == completed))
        .cloned()
        .collect();
    display_tasks(&filtered_tasks, date_filter, status_filter, sort_order);
}

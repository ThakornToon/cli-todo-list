use crate::model::{CalendarFilter, DateFilter, SortOrder, Task, pad_to_width, sort_tasks};

// แสดงรายการที่ผ่านตัวกรอง โดยเรียงสำเนาเพื่อไม่เปลี่ยนลำดับข้อมูลต้นฉบับ
pub fn display_tasks(
    tasks: &[Task],
    filter: DateFilter,
    status_filter: Option<bool>,
    sort_order: SortOrder,
) {
    if tasks.is_empty() {
        println!("\n📭 ไม่พบ Task ตามตัวกรองที่เลือก");
        return;
    }

    // การเรียงบนสำเนาช่วยให้ลำดับในหน่วยความจำและไฟล์ไม่เปลี่ยนจากการดูรายการ
    let mut sorted_tasks: Vec<Task> = tasks.to_vec();
    sort_tasks(&mut sorted_tasks, sort_order);
    let status_label = match status_filter {
        Some(false) => "ยังไม่เสร็จ",
        Some(true) => "เสร็จแล้ว",
        None => "ทั้งหมด",
    };
    println!(
        "\n📋 รายการ Task ({}, สถานะ: {status_label}, เรียง: {})",
        filter.description(),
        sort_order.label()
    );
    // กำหนดความกว้างคอลัมน์เพื่อให้อ่านตารางได้ แม้ข้อความมีอักขระหลายไบต์
    const ID_WIDTH: usize = 4;
    const TITLE_WIDTH: usize = 25;
    const DATE_WIDTH: usize = 20;
    const STATUS_WIDTH: usize = 12;
    println!(
        "{} | {} | {} | {}",
        pad_to_width("ID", ID_WIDTH),
        pad_to_width("ชื่องาน", TITLE_WIDTH),
        pad_to_width("กำหนดเวลา", DATE_WIDTH),
        pad_to_width("สถานะ", STATUS_WIDTH)
    );
    println!(
        "{}",
        "-".repeat(ID_WIDTH + TITLE_WIDTH + DATE_WIDTH + STATUS_WIDTH + 9)
    );

    for task in sorted_tasks {
        let status = if task.completed {
            "✅ เสร็จแล้ว"
        } else {
            "⏳ ยังไม่เสร็จ"
        };
        let formatted_date = task.due_date.format("%Y-%m-%d %H:%M").to_string();
        println!(
            "{} | {} | {} | {}",
            pad_to_width(&task.id.to_string(), ID_WIDTH),
            pad_to_width(&task.title, TITLE_WIDTH),
            pad_to_width(&formatted_date, DATE_WIDTH),
            pad_to_width(status, STATUS_WIDTH)
        );
    }
}

// แสดง Task ทั้งหมดโดยไม่กรอง และเรียงตามกำหนดส่งเพื่อใช้ก่อนแก้ไข/ลบ/เปลี่ยนสถานะ
pub fn display_all_tasks(tasks: &[Task]) {
    display_tasks(
        tasks,
        DateFilter {
            calendar: CalendarFilter::All,
            range: None,
        },
        None,
        SortOrder::DueDate,
    );
}

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, Read, Write};
use unicode_width::UnicodeWidthStr;

// ชื่อไฟล์ JSON ที่เก็บข้อมูล โดยอ้างอิงจากไดเรกทอรีที่ใช้รันโปรแกรม
const FILE_PATH: &str = "todos.json";

// โครงสร้างข้อมูลของงานหนึ่งรายการ
// Serialize/Deserialize ใช้แปลง Task ไป-กลับเป็น JSON สำหรับบันทึกและโหลดไฟล์
// Debug ใช้แสดงรายละเอียด Task ในรูปแบบดีบัก ส่วน Clone ใช้คัดลอก Task
// เช่น สำรองค่าเดิมก่อนแก้ไขหรือลบ เพื่อกู้คืนได้หากบันทึกไฟล์ไม่สำเร็จ
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    // ID ใช้ระบุ Task และไม่เปลี่ยนเมื่อแก้ไขหรือลบรายการอื่น
    id: usize,
    // ชื่องานที่ผู้ใช้กรอก
    title: String,
    // กำหนดส่ง ซึ่งเก็บทั้งวันที่และเวลา
    due_date: NaiveDateTime,
    // true หมายถึงทำเสร็จแล้ว และ false หมายถึงยังไม่เสร็จ
    completed: bool,
}

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
        println!("6. ออกจากโปรแกรม");
        let choice = match prompt_line("กรุณาเลือกเมนู (1-6): ") {
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
            "6" => {
                println!("ม้าที่ว่าเร็วก็ยังแพ้ลา...เพราะลาไปก่อน ✌️");
                break;
            }
            _ => println!("❌ เมนูไม่ถูกต้อง กรุณาเลือก 1-6"),
        }
    }
}

// เพิ่ม Task ใหม่: ตรวจสอบชื่อและวันเวลา สร้าง ID ถัดไป แล้วบันทึกลงไฟล์
fn add_task(tasks: &mut Vec<Task>) {
    let Some(title) = prompt_nonempty_title("📝 ป้อนชื่องาน: ") else {
        return;
    };
    let Some(due_date) = prompt_datetime("📅 วันและเวลา (YYYY-MM-DD HH:MM): ") else {
        return;
    };

    // ใช้ ID สูงสุดที่มีอยู่บวกหนึ่ง ทำให้การลบ Task ไม่ทำให้ ID เดิมถูกนำมาใช้ซ้ำ
    // checked_add ป้องกันการล้นของ usize; במקרהนั้นยกเลิกการเพิ่มอย่างปลอดภัย
    let id: usize = match tasks
        .iter()
        .map(|task| task.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)
    {
        Some(id) => id,
        None => {
            println!("❌ ไม่สามารถเพิ่ม Task ได้ เนื่องจาก ID เต็มแล้ว");
            return;
        }
    };

    let task = Task {
        id,
        title,
        due_date,
        completed: false,
    };

    // เพิ่มรายการในหน่วยความจำก่อนบันทึก; หากบันทึกผิดพลาด ให้นำรายการนั้นกลับออก
    tasks.push(task.clone());
    match save_tasks(tasks) {
        Ok(()) => println!(
            "✅ เพิ่ม Task #{} “{}” กำหนดส่ง {} สำเร็จ",
            task.id,
            task.title,
            task.due_date.format("%Y-%m-%d %H:%M")
        ),
        Err(error) => {
            tasks.pop();
            eprintln!("❌ บันทึก Task ไม่สำเร็จ: {error}");
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

// แสดงรายการที่ผ่านตัวกรอง โดยเรียงสำเนาเพื่อไม่เปลี่ยนลำดับข้อมูลต้นฉบับ
fn display_tasks(
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

// ระดับความละเอียดของการกรองปฏิทิน: ทั้งหมด, ปี, เดือน หรือวัน
// Clone/Copy ทำให้ส่งค่าตัวกรองขนาดเล็กไปใช้ซ้ำได้โดยไม่ต้องย้ายค่าเดิม
#[derive(Clone, Copy)]
enum CalendarFilter {
    All,
    Year(i32),
    Month(i32, u32),
    Day(i32, u32, u32),
}

// ตัวกรองวันที่ประกอบด้วยเงื่อนไขปฏิทิน และช่วงวันแบบรวมวันเริ่มต้น/สิ้นสุด
// Clone/Copy ใช้ส่งตัวกรองไปตรวจหลาย Task และส่งต่อไปแสดงผล โดยยังใช้ค่าเดิมได้
#[derive(Clone, Copy)]
struct DateFilter {
    calendar: CalendarFilter,
    range: Option<(NaiveDate, NaiveDate)>,
}

impl DateFilter {
    // สร้างข้อความภาษาไทยอธิบายตัวกรอง เพื่อนำไปแสดงในหัวตารางรายการ
    fn description(self) -> String {
        if let Some((start, end)) = self.range {
            return format!("{} ถึง {}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d"));
        }
        match self.calendar {
            CalendarFilter::All => "ทุกวัน".to_string(),
            CalendarFilter::Year(year) => format!("ปี {year} ทุกเดือน"),
            CalendarFilter::Month(year, month) => format!("ปี {year} เดือน {month} ทุกวัน"),
            CalendarFilter::Day(year, month, day) => {
                format!("วันที่ {year:04}-{month:02}-{day:02}")
            }
        }
    }
}

// ตัวเลือกคอลัมน์ที่ใช้เรียง Task ในหน้ารายการ
// Clone/Copy ทำให้ส่งตัวเลือกลำดับการเรียงไปยังฟังก์ชันต่าง ๆ ได้โดยไม่ย้ายค่า
#[derive(Clone, Copy)]
enum SortOrder {
    DueDate,
    Status,
    Title,
}

impl SortOrder {
    // คืนชื่อภาษาไทยสำหรับแสดงว่ารายการถูกเรียงตามอะไร
    fn label(self) -> &'static str {
        match self {
            Self::DueDate => "กำหนดเวลา",
            Self::Status => "สถานะ",
            Self::Title => "ชื่องาน",
        }
    }
}

// เรียง Task ตามตัวเลือก และใช้ข้อมูลสำรอง (เช่น ID หรือกำหนดส่ง)
// เพื่อให้ผลการเรียงมีลำดับแน่นอนเมื่อค่าหลักเท่ากัน
fn sort_tasks(tasks: &mut [Task], order: SortOrder) {
    tasks.sort_by(|a, b| match order {
        // เรียงตามกำหนดส่งก่อน แล้วตาม ID เพื่อให้ลำดับแน่นอนเมื่อวันเวลาเท่ากัน
        SortOrder::DueDate => a.due_date.cmp(&b.due_date).then(a.id.cmp(&b.id)),
        // เรียงตามสถานะก่อน (ยังไม่เสร็จก่อนเสร็จแล้ว) แล้วตามกำหนดส่ง
        SortOrder::Status => a
            .completed
            .cmp(&b.completed)
            .then(a.due_date.cmp(&b.due_date)),
        // เรียงตามชื่องานแบบไม่สนตัวพิมพ์เล็ก/ใหญ่ แล้วตามกำหนดส่ง และตาม ID
        SortOrder::Title => a
            .title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then(a.due_date.cmp(&b.due_date))
            .then(a.id.cmp(&b.id)),
    });
}

// เติมช่องว่างด้านขวาให้ค่ามีความกว้างตามที่แสดงจริง โดยไม่ตัดข้อความที่ยาวเกิน
fn pad_to_width(value: &str, width: usize) -> String {
    let display_width = UnicodeWidthStr::width(value);
    format!("{value}{}", " ".repeat(width.saturating_sub(display_width)))
}

// ตรวจว่า Task ผ่านทั้งตัวกรองปฏิทินและช่วงวันที่ (ถ้ามี) หรือไม่
fn task_matches_filter(task: &Task, filter: DateFilter) -> bool {
    let calendar_matches = match filter.calendar {
        CalendarFilter::All => true,
        CalendarFilter::Year(year) => task.due_date.year() == year,
        CalendarFilter::Month(year, month) => {
            task.due_date.year() == year && task.due_date.month() == month
        }
        CalendarFilter::Day(year, month, day) => {
            task.due_date.year() == year
                && task.due_date.month() == month
                && task.due_date.day() == day
        }
    };
    let range_matches = filter
        .range
        .is_none_or(|(start, end)| (start..=end).contains(&task.due_date.date()));
    calendar_matches && range_matches
}

// ถามระดับตัวกรองปี/เดือน/วัน โดยใช้ * เพื่อเลือก "ทั้งหมด" ในแต่ละระดับ
fn prompt_calendar_filter() -> Option<DateFilter> {
    let year: Option<u32> = prompt_number("📅 เลือกปี (YYYY หรือ * ทุกปี): ", 1, 9999)?;
    let calendar = match year {
        None => CalendarFilter::All,
        Some(year) => {
            let month: Option<u32> = prompt_number("📅 เลือกเดือน (1-12 หรือ * ทุกเดือน): ", 1, 12)?;
            match month {
                None => CalendarFilter::Year(year as i32),
                Some(month) => loop {
                    let day: Option<u32> = prompt_number("📅 เลือกวัน (1-31 หรือ * ทุกวัน): ", 1, 31)?;
                    match day {
                        None => break CalendarFilter::Month(year as i32, month),
                        Some(day) if NaiveDate::from_ymd_opt(year as i32, month, day).is_some() => {
                            break CalendarFilter::Day(year as i32, month, day);
                        }
                        Some(_) => {
                            println!("❌ วันที่นี้ไม่มีอยู่ในเดือนที่เลือก");
                        }
                    }
                },
            }
        }
    };
    Some(DateFilter {
        calendar,
        range: None,
    })
}

// อ่านจำนวนเต็มในช่วงที่กำหนด หรือคืน None เมื่อผู้ใช้เลือก *; Option ชั้นนอก
// ใช้แยกกรณียกเลิก/จบอินพุตออกจากการเลือกทั้งหมด
fn prompt_number(prompt: &str, min: u32, max: u32) -> Option<Option<u32>> {
    loop {
        let input: String = prompt_value(prompt)?;  // คืน None เมื่อผู้ใช้สั่งกลับหรืออินพุตสิ้นสุด
        let input: &str = input.trim();
        if input == "*" {
            return Some(None);
        }
        // ตรวจสอบว่าค่าที่กรอกเป็นตัวเลขและอยู่ในช่วงที่กำหนด
        match input.parse::<u32>() {
            Ok(value) if (min..=max).contains(&value) => return Some(Some(value)),
            _ => println!("❌ กรุณากรอกค่าระหว่าง {min}-{max} หรือ * เพื่อเลือกทั้งหมด"),
        }
    }
}

// รับช่วงวันที่และตรวจว่าวันเริ่มต้นไม่อยู่หลังวันสิ้นสุด
fn prompt_date_range() -> Option<(NaiveDate, NaiveDate)> {
    loop {
        let start = prompt_date("📅 วันที่เริ่มต้น (YYYY-MM-DD): ")?;
        let end = prompt_date("📅 วันที่สิ้นสุด (YYYY-MM-DD): ")?;
        if start <= end {
            return Some((start, end));
        }
        println!("❌ วันที่สิ้นสุดต้องไม่อยู่ก่อนวันที่เริ่มต้น");
    }
}

// อ่านวันที่ในรูปแบบ YYYY-MM-DD และถามซ้ำจนกว่าจะถูกต้องหรือยกเลิก
fn prompt_date(prompt: &str) -> Option<NaiveDate> {
    loop {
        let input = prompt_value(prompt)?;
        match NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d") {
            Ok(date) => return Some(date),
            Err(_) => println!("❌ รูปแบบวันที่ไม่ถูกต้อง กรุณาใช้ YYYY-MM-DD"),
        }
    }
}

// อ่านวันและเวลาในรูปแบบ YYYY-MM-DD HH:MM และตรวจความถูกต้องก่อนคืนค่า
fn prompt_datetime(prompt: &str) -> Option<NaiveDateTime> {
    loop {
        let input = prompt_value(prompt)?;
        match NaiveDateTime::parse_from_str(input.trim(), "%Y-%m-%d %H:%M") {
            Ok(datetime) => return Some(datetime),
            Err(_) => println!("❌ รูปแบบวันเวลาไม่ถูกต้อง กรุณาใช้ YYYY-MM-DD HH:MM"),
        }
    }
}

// อ่านชื่องานและไม่ยอมรับข้อความว่างหลังตัดช่องว่างหัวท้าย
fn prompt_nonempty_title(prompt: &str) -> Option<String> {
    loop {
        let input = prompt_value(prompt)?;  // คืน None เมื่อผู้ใช้สั่งกลับหรืออินพุตสิ้นสุด
        let title = input.trim();
        if !title.is_empty() {
            return Some(title.to_string());
        }
        println!("❌ ชื่องานห้ามว่าง กรุณาลองใหม่");
    }
}

// ผลลัพธ์การอ่านอินพุต: ได้ข้อความ, ผู้ใช้สั่งกลับ หรืออินพุตสิ้นสุด/อ่านไม่ได้
enum PromptInput {
    Value(String),
    Back,
    EndOfInput,
}

// แสดง prompt และอ่านข้อความหนึ่งบรรทัด โดยคำสั่ง b ใช้ยกเลิกขั้นตอนปัจจุบัน
fn prompt_line(prompt: &str) -> PromptInput {
    print!("{prompt} [b กลับเมนูหลัก] ");

    // print! อาจยังไม่แสดงทันทีจึง flush stdout เพื่อให้ prompt แสดงก่อนอ่านอินพุต
    if let Err(error) = io::stdout().flush() {
        eprintln!("❌ แสดงข้อความไม่ได้: {error}");
        return PromptInput::EndOfInput;
    }

    let mut input: String = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => PromptInput::EndOfInput,  // ไม่มีไบต์ให้อ่าน 
        Ok(_) if is_back_command(&input) => {
            println!("↩️ ยกเลิกขั้นตอนปัจจุบัน กลับเมนูหลัก");
            PromptInput::Back
        }
        Ok(_) => PromptInput::Value(input),
        Err(error) => {
            eprintln!("❌ อ่านข้อมูลไม่ได้: {error}");
            PromptInput::EndOfInput
        }
    }
}

// ตรวจคำสั่งกลับเมนูโดยไม่สนตัวพิมพ์เล็ก/ใหญ่และละช่องว่างหัวท้าย
fn is_back_command(input: &str) -> bool {
    input.trim().eq_ignore_ascii_case("b")
}

// ตัวช่วยสำหรับขั้นตอนที่ต้องการข้อความอย่างเดียว โดยแปลงการกลับ/จบอินพุตเป็น None
fn prompt_value(prompt: &str) -> Option<String> {
    // ยุบผลลัพธ์ Back กับ EndOfInput จาก prompt_line(prompt) ให้เป็น None ส่วนข้อความปกติเป็น Some(input)
    match prompt_line(prompt) {
        PromptInput::Value(input) => Some(input),
        PromptInput::Back | PromptInput::EndOfInput => None,
    }
}

// กลับสถานะของ Task หลายรายการตาม ID ที่ระบุ และบันทึกผลลงไฟล์
fn toggle_task_status(tasks: &mut [Task]) {
    display_all_tasks(tasks);
    if tasks.is_empty() {
        return;
    }

    let Some(ids) = prompt_task_ids("📌 ป้อน ID ที่ต้องการกลับสถานะ (เช่น 1, 2, 3): ") else {
        return;
    };

    // เปลี่ยนเฉพาะ ID ที่พบ; ถ้าไม่มีรายการตรงกันก็ไม่ต้องเขียนไฟล์
    let found_ids = toggle_tasks(tasks, &ids);  // คืนชุด ID ที่พบจริง
    if found_ids.is_empty() {
        println!("❌ ไม่พบ Task ตาม ID ที่ระบุ");
        return;
    }

    match save_tasks(tasks) {
        Ok(()) => {
            println!("✅ อัปเดตสถานะแล้ว:");
            for task in tasks.iter().filter(|task| found_ids.contains(&task.id)) {
                println!(
                    "  • #{} {} → {}",
                    task.id,
                    task.title,
                    if task.completed {
                        "เสร็จแล้ว"
                    } else {
                        "ยังไม่เสร็จ"
                    }
                );
            }
            // แสดง ID ที่ไม่พบเพื่อให้ผู้ใช้ทราบ
            let missing_ids: Vec<_> = ids.difference(&found_ids).collect();
            if !missing_ids.is_empty() {
                println!("⚠️ ไม่พบ ID: {missing_ids:?}");
            }
        }
        Err(error) => {
            // ย้อนสถานะกลับด้วยการ toggle ซ้ำ เพื่อให้ข้อมูลในหน่วยความจำตรงกับไฟล์
            toggle_tasks(tasks, &found_ids);
            eprintln!("❌ บันทึกสถานะ Task ไม่สำเร็จ: {error}");
        }
    }
}

// สลับสถานะของ Task ที่ ID อยู่ในชุดที่ระบุ และคืนชุด ID ที่พบจริง
fn toggle_tasks(tasks: &mut [Task], ids: &BTreeSet<usize>) -> BTreeSet<usize> {
    let mut found_ids = BTreeSet::new();
    for task in tasks.iter_mut().filter(|task| ids.contains(&task.id)) {
        task.completed = !task.completed;
        found_ids.insert(task.id);
    }
    found_ids
}

// ลบ Task ตาม ID หลายรายการหลังแสดงตัวอย่างและได้รับการยืนยันจากผู้ใช้
fn delete_task(tasks: &mut Vec<Task>) {
    display_all_tasks(tasks);
    if tasks.is_empty() {
        return;
    }

    let Some(ids) = prompt_task_ids("🗑️ ป้อน ID ที่ต้องการลบ (เช่น 1, 2, 3): ")
    else {
        return;
    };

    // ตรวจสอบว่ามี Task ที่ตรงกับ ID ที่ระบุจริงหรือไม่ และเก็บ ID ที่พบไว้ใน found_ids
    let found_ids: BTreeSet<_> = tasks
        .iter()
        .filter(|task| ids.contains(&task.id))
        .map(|task| task.id)
        .collect();
    if found_ids.is_empty() {
        println!("❌ ไม่พบ Task ตาม ID ที่ระบุ");
        return;
    }

    // แสดงรายการที่จะลบเพื่อให้ผู้ใช้ยืนยันก่อนลบจริง
    let to_delete: Vec<_> = tasks
        .iter()
        .filter(|task| found_ids.contains(&task.id))
        .cloned()
        .collect();
    println!("รายการที่จะลบ:");
    for task in &to_delete {
        println!("  • #{} {}", task.id, task.title);
    }
    
    // ขอการยืนยันจากผู้ใช้ก่อนลบจริง โดยรองรับ y/yes/ใช่ และ n/no/ไม่
    loop {
        let Some(answer) = prompt_value("ยืนยันการลบ? (y/n)") else {
            return;
        };
        match answer.trim().to_lowercase().as_str() {
            "y" | "yes" | "ใช่" => break,
            "n" | "no" | "ไม่" => {
                println!("ยกเลิกการลบแล้วจัฟ");
                return;
            }
            _ => println!("❌ กรุณาตอบ y หรือ n"),
        }
    }

    // สำรองรายการเดิมไว้กู้คืน หากการบันทึกหลังลบไม่สำเร็จ
    let original_tasks = tasks.clone();
    // ลบ Task ที่ตรงกับ ID ที่ระบุออกจาก tasks โดยใช้ retain
    tasks.retain(|task| !ids.contains(&task.id));

    match save_tasks(tasks) {
        Ok(()) => {
            let missing_ids: Vec<_> = ids.difference(&found_ids).collect();
            println!(
                "✅ ลบ Task แล้ว {} รายการ:",
                original_tasks.len() - tasks.len()
            );
            for task in to_delete {
                println!("  • #{} {}", task.id, task.title);
            }
            if !missing_ids.is_empty() {
                println!("⚠️ ไม่พบ ID: {missing_ids:?}");
            }
        }
        Err(error) => {
            // คืนข้อมูลเดิมในหน่วยความจำเมื่อไม่สามารถบันทึกการลบลงไฟล์ได้
            *tasks = original_tasks;
            eprintln!("❌ บันทึกการลบ Task ไม่สำเร็จ: {error}");
        }
    }
}

// แก้ไขชื่อและ/หรือกำหนดส่งของ Task หนึ่งรายการ; ช่องว่างหมายถึงคงค่าเดิม
fn edit_task(tasks: &mut [Task]) {
    display_all_tasks(tasks);
    if tasks.is_empty() {
        return;
    }
    let Some(id) = prompt_existing_task_id(tasks, "✏️ ป้อน ID ของ Task ที่ต้องการแก้ไข: ")
    else {
        return;
    };
    let Some(index) = tasks.iter().position(|task| task.id == id) else {
        eprintln!("❌ ไม่พบ Task ID {id}");
        return;
    };
    // เก็บสำเนาก่อนแก้ไขเพื่อใช้ย้อนกลับ หากบันทึกข้อมูลใหม่ไม่สำเร็จ
    let original: Task = tasks[index].clone();
    println!("เว้นว่างเพื่อคงค่าเดิม");

    let Some(input) = prompt_value(&format!("ชื่องาน [{}]", original.title)) else {
        return;
    };
    let title: String = if input.trim().is_empty() {
        original.title.clone()
    } else {
        input.trim().to_string()
    };
    let due_date = loop {
        let Some(input) = prompt_value(&format!(
            "กำหนดเวลา [{}] (YYYY-MM-DD HH:MM)",
            original.due_date.format("%Y-%m-%d %H:%M")
        )) else {
            return;
        };
        let input = input.trim();
        if input.is_empty() {
            break original.due_date;
        }
        match NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M") {
            Ok(datetime) => break datetime,
            Err(_) => println!("❌ รูปแบบวันเวลาไม่ถูกต้อง กรุณาใช้ YYYY-MM-DD HH:MM"),
        }
    };
    tasks[index].title = title;
    tasks[index].due_date = due_date;
    match save_tasks(tasks) {
        Ok(()) => println!(
            "✅ แก้ไข Task #{} แล้ว: “{}” กำหนดส่ง {}",
            tasks[index].id,
            tasks[index].title,
            tasks[index].due_date.format("%Y-%m-%d %H:%M")
        ),
        Err(error) => {
            // คืน Task เดิม เพื่อไม่ให้หน่วยความจำแสดงค่าที่ไม่ได้ถูกบันทึก
            tasks[index] = original;
            eprintln!("❌ บันทึกการแก้ไขไม่สำเร็จ: {error}");
        }
    }
}

// ขอ ID ซ้ำจนกว่าจะกรอกตัวเลขที่ตรงกับ Task ซึ่งมีอยู่จริง
fn prompt_existing_task_id(tasks: &[Task], prompt: &str) -> Option<usize> {
    loop {
        let input = prompt_value(prompt)?;
        match input.trim().parse::<usize>() {
            Ok(id) if tasks.iter().any(|task| task.id == id) => return Some(id),
            Ok(id) => println!("❌ ไม่พบ Task ID {id} กรุณาลองใหม่"),
            Err(_) => println!("❌ กรุณากรอก ID เป็นตัวเลข"),
        }
    }
}

// ขอชุด ID ซ้ำจนกว่าจะอ่านและแปลงรายการ ID ได้ หรือผู้ใช้ยกเลิก
fn prompt_task_ids(prompt: &str) -> Option<BTreeSet<usize>> {
    loop {
        let input = prompt_value(prompt)?;
        match parse_task_ids(&input) {
            Ok(ids) => return Some(ids),
            Err(()) => println!("❌ กรุณากรอก ID เป็นตัวเลข คั่นด้วยจุลภาคหรือเว้นวรรค"),
        }
    }
}

// แสดง Task ทั้งหมดโดยไม่กรอง และเรียงตามกำหนดส่งเพื่อใช้ก่อนแก้ไข/ลบ/เปลี่ยนสถานะ
fn display_all_tasks(tasks: &[Task]) {
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

// แปลงข้อความ ID ที่คั่นด้วยจุลภาคหรือช่องว่างเป็นชุดตัวเลขที่ไม่ซ้ำและเรียงลำดับ
fn parse_task_ids(input: &str) -> Result<BTreeSet<usize>, ()> {
    let mut ids = BTreeSet::new();
    for group in input.trim().split(',') {
        let tokens: Vec<&str> = group.split_whitespace().collect();
        if tokens.is_empty() {
            return Err(());
        }
        for token in tokens {
            // แปลงแต่ละข้อความเป็น usize แล้วเพิ่มลง ids ถ้าแปลงไม่ได้ให้เปลี่ยนข้อผิดพลาดเป็น () และ ? ส่ง Err(()) ออกจากฟังก์ชันทันที
            ids.insert(token.parse::<usize>().map_err(|_| ())?);
        }
    }
    if ids.is_empty() { Err(()) } else { Ok(ids) }
}

// ตรวจสอบงานที่ยังไม่เสร็จ และแสดงกลุ่มงานเลยกำหนดหรือถึงกำหนดภายใน 24 ชั่วโมง
fn check_notifications(tasks: &[Task]) {
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

// serialize รายการเป็น JSON ที่มีการจัดรูปแบบอ่านง่าย แล้วเขียนทับไฟล์ข้อมูล
fn save_tasks(tasks: &[Task]) -> io::Result<()> {
    // ใช้ to_string_pretty เพื่อให้ไฟล์ JSON อ่านง่ายสำหรับผู้ใช้ และ map_err เพื่อแปลงข้อผิดพลาดเป็น io::Error
    let json = serde_json::to_string_pretty(tasks)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    
    // สร้างไฟล์ใหม่ทับไฟล์เดิม หากเกิดข้อผิดพลาดจะไม่ลบไฟล์เดิม
    let mut file = File::create(FILE_PATH)?;
    file.write_all(json.as_bytes())
}

// โหลดรายการ JSON ไฟล์ที่ยังไม่มีถือเป็นรายการว่าง ส่วนข้อผิดพลาดอื่นจะถูกรายงาน
fn load_tasks() -> io::Result<Vec<Task>> {
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

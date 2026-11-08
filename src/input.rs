use std::collections::BTreeSet;
use std::io::{self, Write};

use chrono::{NaiveDate, NaiveDateTime};

use crate::model::{CalendarFilter, DateFilter, Task};

// ถามระดับตัวกรองปี/เดือน/วัน โดยใช้ * เพื่อเลือก "ทั้งหมด" ในแต่ละระดับ
pub fn prompt_calendar_filter() -> Option<DateFilter> {
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
pub fn prompt_number(prompt: &str, min: u32, max: u32) -> Option<Option<u32>> {
    loop {
        let input: String = prompt_value(prompt)?; // คืน None เมื่อผู้ใช้สั่งกลับหรืออินพุตสิ้นสุด
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
pub fn prompt_date_range() -> Option<(NaiveDate, NaiveDate)> {
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
pub fn prompt_date(prompt: &str) -> Option<NaiveDate> {
    loop {
        let input = prompt_value(prompt)?;
        match NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d") {
            Ok(date) => return Some(date),
            Err(_) => println!("❌ รูปแบบวันที่ไม่ถูกต้อง กรุณาใช้ YYYY-MM-DD"),
        }
    }
}

// อ่านวันและเวลาในรูปแบบ YYYY-MM-DD HH:MM และตรวจความถูกต้องก่อนคืนค่า
pub fn prompt_datetime(prompt: &str) -> Option<NaiveDateTime> {
    loop {
        let input = prompt_value(prompt)?;
        match NaiveDateTime::parse_from_str(input.trim(), "%Y-%m-%d %H:%M") {
            Ok(datetime) => return Some(datetime),
            Err(_) => println!("❌ รูปแบบวันเวลาไม่ถูกต้อง กรุณาใช้ YYYY-MM-DD HH:MM"),
        }
    }
}

// อ่านชื่องานและไม่ยอมรับข้อความว่างหลังตัดช่องว่างหัวท้าย
pub fn prompt_nonempty_title(prompt: &str) -> Option<String> {
    loop {
        let input = prompt_value(prompt)?; // คืน None เมื่อผู้ใช้สั่งกลับหรืออินพุตสิ้นสุด
        let title = input.trim();
        if !title.is_empty() {
            return Some(title.to_string());
        }
        println!("❌ ชื่องานห้ามว่าง กรุณาลองใหม่");
    }
}

// ผลลัพธ์การอ่านอินพุต: ได้ข้อความ, ผู้ใช้สั่งกลับ หรืออินพุตสิ้นสุด/อ่านไม่ได้
pub enum PromptInput {
    Value(String),
    Back,
    EndOfInput,
}

// แสดง prompt และอ่านข้อความหนึ่งบรรทัด โดยคำสั่ง b ใช้ยกเลิกขั้นตอนปัจจุบัน
pub fn prompt_line(prompt: &str) -> PromptInput {
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
pub fn prompt_value(prompt: &str) -> Option<String> {
    // ยุบผลลัพธ์ Back กับ EndOfInput จาก prompt_line(prompt) ให้เป็น None ส่วนข้อความปกติเป็น Some(input)
    match prompt_line(prompt) {
        PromptInput::Value(input) => Some(input),
        PromptInput::Back | PromptInput::EndOfInput => None,
    }
}

// ขอ ID ซ้ำจนกว่าจะกรอกตัวเลขที่ตรงกับ Task ซึ่งมีอยู่จริง
pub fn prompt_existing_task_id(tasks: &[Task], prompt: &str) -> Option<usize> {
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
pub fn prompt_task_ids(prompt: &str) -> Option<BTreeSet<usize>> {
    loop {
        let input = prompt_value(prompt)?;
        match parse_task_ids(&input) {
            Ok(ids) => return Some(ids),
            Err(()) => println!("❌ กรุณากรอก ID เป็นตัวเลข คั่นด้วยจุลภาคหรือเว้นวรรค"),
        }
    }
}

// แปลงข้อความ ID ที่คั่นด้วยจุลภาคหรือช่องว่างเป็นชุดตัวเลขที่ไม่ซ้ำและเรียงลำดับ
pub fn parse_task_ids(input: &str) -> Result<BTreeSet<usize>, ()> {
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

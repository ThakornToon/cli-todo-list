use chrono::{Datelike, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;

// โครงสร้างข้อมูลของงานหนึ่งรายการ
// Serialize/Deserialize ใช้แปลง Task ไป-กลับเป็น JSON สำหรับบันทึกและโหลดไฟล์
// Debug ใช้แสดงรายละเอียด Task ในรูปแบบดีบัก ส่วน Clone ใช้คัดลอก Task
// เช่น สำรองค่าเดิมก่อนแก้ไขหรือลบ เพื่อกู้คืนได้หากบันทึกไฟล์ไม่สำเร็จ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    // ID ใช้ระบุ Task และไม่เปลี่ยนเมื่อแก้ไขหรือลบรายการอื่น
    pub id: usize,
    // ชื่องานที่ผู้ใช้กรอก
    pub title: String,
    // กำหนดส่ง ซึ่งเก็บทั้งวันที่และเวลา
    pub due_date: NaiveDateTime,
    // true หมายถึงทำเสร็จแล้ว และ false หมายถึงยังไม่เสร็จ
    pub completed: bool,
}

// ระดับความละเอียดของการกรองปฏิทิน: ทั้งหมด, ปี, เดือน หรือวัน
// Clone/Copy ทำให้ส่งค่าตัวกรองขนาดเล็กไปใช้ซ้ำได้โดยไม่ต้องย้ายค่าเดิม
#[derive(Clone, Copy)]
pub enum CalendarFilter {
    All,
    Year(i32),
    Month(i32, u32),
    Day(i32, u32, u32),
}

// ตัวกรองวันที่ประกอบด้วยเงื่อนไขปฏิทิน และช่วงวันแบบรวมวันเริ่มต้น/สิ้นสุด
// Clone/Copy ใช้ส่งตัวกรองไปตรวจหลาย Task และส่งต่อไปแสดงผล โดยยังใช้ค่าเดิมได้
#[derive(Clone, Copy)]
pub struct DateFilter {
    pub calendar: CalendarFilter,
    pub range: Option<(NaiveDate, NaiveDate)>,
}

impl DateFilter {
    // สร้างข้อความภาษาไทยอธิบายตัวกรอง เพื่อนำไปแสดงในหัวตารางรายการ
    pub fn description(self) -> String {
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
pub enum SortOrder {
    DueDate,
    Status,
    Title,
}

impl SortOrder {
    // คืนชื่อภาษาไทยสำหรับแสดงว่ารายการถูกเรียงตามอะไร
    pub fn label(self) -> &'static str {
        match self {
            Self::DueDate => "กำหนดเวลา",
            Self::Status => "สถานะ",
            Self::Title => "ชื่องาน",
        }
    }
}

// เรียง Task ตามตัวเลือก และใช้ข้อมูลสำรอง (เช่น ID หรือกำหนดส่ง)
// เพื่อให้ผลการเรียงมีลำดับแน่นอนเมื่อค่าหลักเท่ากัน
pub fn sort_tasks(tasks: &mut [Task], order: SortOrder) {
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
pub fn pad_to_width(value: &str, width: usize) -> String {
    let display_width = UnicodeWidthStr::width(value);
    format!("{value}{}", " ".repeat(width.saturating_sub(display_width)))
}

// ตรวจว่า Task ผ่านทั้งตัวกรองปฏิทินและช่วงวันที่ (ถ้ามี) หรือไม่
pub fn task_matches_filter(task: &Task, filter: DateFilter) -> bool {
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

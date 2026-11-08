use std::collections::BTreeSet;

use chrono::NaiveDateTime;

use crate::display::display_all_tasks;
use crate::input::{prompt_datetime, prompt_existing_task_id, prompt_nonempty_title, prompt_task_ids, prompt_value,};
use crate::model::Task;
use crate::storage::save_tasks;

// เพิ่ม Task ใหม่: ตรวจสอบชื่อและวันเวลา สร้าง ID ถัดไป แล้วบันทึกลงไฟล์
pub fn add_task(tasks: &mut Vec<Task>) {
    let Some(title) = prompt_nonempty_title("📝 ป้อนชื่องาน: ") else {
        return;
    };
    let Some(due_date) = prompt_datetime("📅 วันและเวลา (YYYY-MM-DD HH:MM): ")
    else {
        return;
    };

    // ใช้ ID สูงสุดที่มีอยู่บวกหนึ่ง ทำให้การเพิ่ม Task ไม่ทำให้ ID เดิมถูกนำมาใช้ซ้ำถ้า Task ถูกลบไปแล้ว
    // checked_add ป้องกันการล้นของ usize; ในกรณีนี้ยกเลิกการเพิ่มอย่างปลอดภัย
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

// กลับสถานะของ Task หลายรายการตาม ID ที่ระบุ และบันทึกผลลงไฟล์
pub fn toggle_task_status(tasks: &mut [Task]) {
    display_all_tasks(tasks);
    if tasks.is_empty() {
        return;
    }

    let Some(ids) = prompt_task_ids("📌 ป้อน ID ที่ต้องการกลับสถานะ (เช่น 1, 2, 3): ")
    else {
        return;
    };

    // เปลี่ยนเฉพาะ ID ที่พบ; ถ้าไม่มีรายการตรงกันก็ไม่ต้องเขียนไฟล์
    let found_ids = toggle_tasks(tasks, &ids); // คืนชุด ID ที่พบจริง
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
pub fn delete_task(tasks: &mut Vec<Task>) {
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
pub fn edit_task(tasks: &mut [Task]) {
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

// จัด ID ใหม่เป็น 1..N ตามลำดับ Task ในรายการ และบันทึกผลลงไฟล์
pub fn normalize_task_ids(tasks: &mut [Task]) {
    if tasks.is_empty() {
        println!("📭 ยังไม่มี Task ให้จัดเรียง ID");
        return;
    }

    println!("ID จะถูกกำหนดใหม่ตามลำดับรายการปัจจุบัน:");
    let mut changes = Vec::new();
    for (index, task) in tasks.iter().enumerate() {
        let new_id: usize = index + 1;
        if task.id != new_id {
            println!("  • #{} → #{} {}", task.id, new_id, task.title);
            changes.push((task.id, new_id));
        }
    }

    if changes.is_empty() {
        println!("✅ ID เรียงต่อเนื่องอยู่แล้ว ไม่ต้องเปลี่ยนแปลง");
        return;
    }

    // ขอการยืนยันจากผู้ใช้ก่อนเปลี่ยน ID จริง โดยรองรับ y/yes/ใช่ และ n/no/ไม่
    loop {
        let Some(answer) = prompt_value("ยืนยันการจัดเรียง ID ใหม่? (y/n)")
        else {
            return;
        };
        match answer.trim().to_lowercase().as_str() {
            "y" | "yes" | "ใช่" => break,
            "n" | "no" | "ไม่" => {
                println!("ยกเลิกการจัดเรียง ID แล้ว");
                return;
            }
            _ => println!("❌ กรุณาตอบ y หรือ n"),
        }
    }

    let original_tasks: Vec<Task> = tasks.to_vec(); // สำรองรายการเดิมไว้กู้คืน หากการบันทึกหลังเปลี่ยน ID ไม่สำเร็จ
    for (index, task) in tasks.iter_mut().enumerate() {
        task.id = index + 1;
    }

    match save_tasks(tasks) {
        Ok(()) => println!("✅ จัดเรียง ID ใหม่และบันทึกข้อมูลแล้ว"),
        Err(error) => {
            tasks.clone_from_slice(&original_tasks);
            eprintln!("❌ บันทึกการจัดเรียง ID ไม่สำเร็จ: {error}");
        }
    }
}

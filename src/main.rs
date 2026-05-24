use std::{
    fs,
    io,
    path::Path,
    process,
};

const MONEY_OFFSET: u64 = 0x011B;
const MONEY_SIZE: usize = 2;
const MIN_FILE_SIZE: u64 = MONEY_OFFSET + MONEY_SIZE as u64;
const MAX_MONEY: u32 = 65535;

fn slot_to_filename(slot: &str) -> Result<String, String> {
    let upper = slot.to_uppercase();
    match upper.as_str() {
        "1" | "2" | "3" | "4" | "5" => Ok(format!("SAVE.ZA{}", upper)),
        "Q" => Ok("SAVE.ZAQ".to_string()),
        _ => Err(format!("无效的存档编号: \"{}\"（可用: 1,2,3,4,5,Q）", slot)),
    }
}

fn print_usage() {
    eprintln!("用法: swda_money_editor.exe <存档编号> [金额]");
    eprintln!("说明: 查看或修改轩辕剑外传枫之舞存档的金钱");
    eprintln!("      存档编号: 1, 2, 3, 4, 5, Q");
    eprintln!("      金额范围: 0 ~ 65535（不填则只显示当前金钱）");
}

fn read_current_money(path: &str) -> io::Result<u32> {
    let data = fs::read(path)?;
    if (data.len() as u64) < MIN_FILE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "文件太小",
        ));
    }
    let lo = data[MONEY_OFFSET as usize] as u32;
    let hi = data[MONEY_OFFSET as usize + 1] as u32;
    Ok(lo | (hi << 8))
}

fn backup_file(path: &str) -> io::Result<()> {
    let bak_path = format!("{}.bak", path);
    if Path::new(&bak_path).exists() {
        println!("  备份已存在: {}", bak_path);
        return Ok(());
    }
    fs::copy(path, &bak_path)?;
    println!("  已备份: {} → {}", path, bak_path);
    Ok(())
}

fn modify_money(path: &str, money: u32) -> io::Result<()> {
    let mut data = fs::read(path)?;

    let lo = (money & 0xFF) as u8;
    let hi = ((money >> 8) & 0xFF) as u8;

    data[MONEY_OFFSET as usize] = lo;
    data[MONEY_OFFSET as usize + 1] = hi;

    fs::write(path, &data)?;
    Ok(())
}

fn verify(path: &str, expected: u32) -> io::Result<bool> {
    match read_current_money(path) {
        Ok(actual) => Ok(actual == expected),
        Err(_) => Ok(false),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // 参数太少
    if args.len() < 2 || args.len() > 3 {
        eprintln!("错误: 参数数量不正确");
        eprintln!();
        print_usage();
        process::exit(1);
    }

    // 解析编号
    let filename = match slot_to_filename(&args[1]) {
        Ok(f) => f,
        Err(msg) => {
            eprintln!("错误: {}", msg);
            process::exit(1);
        }
    };

    // 检查存档是否存在
    if !Path::new(&filename).exists() {
        eprintln!("错误: 存档不存在: {}", filename);
        process::exit(1);
    }

    // 读取当前金钱
    let current = match read_current_money(&filename) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("错误: 无法读取存档 {}: {}", filename, e);
            process::exit(1);
        }
    };

    if args.len() == 2 {
        // 仅显示
        println!("存档 {}: {} 文钱", filename, current);
    } else {
        // 修改
        let money_str = &args[2];
        let money: u32 = match money_str.parse() {
            Ok(m) => m,
            Err(_) => {
                eprintln!("错误: 金额 \"{}\" 不是有效的整数", money_str);
                process::exit(1);
            }
        };

        if money > MAX_MONEY {
            eprintln!("错误: 金额 {} 超出上限 {}（16位无符号最大值）", money, MAX_MONEY);
            process::exit(1);
        }

        println!("存档 {}: {} 文钱 → {} 文钱", filename, current, money);

        // 备份
        if let Err(e) = backup_file(&filename) {
            eprintln!("备份失败: {}", e);
            process::exit(1);
        }

        // 修改
        if let Err(e) = modify_money(&filename, money) {
            eprintln!("修改失败: {}", e);
            process::exit(1);
        }

        // 验证
        match verify(&filename, money) {
            Ok(true) => {
                println!("  [OK] 修改成功");
            }
            Ok(false) => {
                eprintln!("  [FAIL] 验证失败，写入结果与预期不符");
                process::exit(1);
            }
            Err(e) => {
                eprintln!("验证过程出错: {}", e);
                process::exit(1);
            }
        }
    }
}

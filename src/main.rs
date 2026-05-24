use std::{
    fs,
    io,
    path::Path,
    process,
};

// ============================================================
// 存档金钱修改（原有功能）
// ============================================================

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

fn read_current_money(path: &str) -> io::Result<u32> {
    let data = fs::read(path)?;
    if (data.len() as u64) < MIN_FILE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "文件太小"));
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

fn verify_money(path: &str, expected: u32) -> io::Result<bool> {
    match read_current_money(path) {
        Ok(actual) => Ok(actual == expected),
        Err(_) => Ok(false),
    }
}

// ============================================================
// EXE 补丁功能
// ============================================================

#[derive(Clone, Copy)]
struct PatchDef {
    /// 命令行标识，如 "fast-level"
    id: &'static str,
    /// 显示名称
    name: &'static str,
    /// 目标 exe 文件名
    exe: &'static str,
    /// 搜索特征串（在未修改的 exe 中定位）
    locate: &'static [u8],
    /// locate 中开始替换的偏移
    offset: usize,
    /// 原始字节
    original: &'static [u8],
    /// 补丁字节（NOP 序列）
    patched: &'static [u8],
}

const PATCHES: &[PatchDef] = &[
    // ---- FIG.EXE ----
    PatchDef {
        id: "fast-level",
        name: "快速升级",
        exe: "FIG.EXE",
        locate: &[0x29, 0x44, 0x39, 0xA1, 0xAC],
        offset: 0,
        original: &[0x29, 0x44, 0x39],
        patched: &[0x90, 0x90, 0x90],
    },
    PatchDef {
        id: "hp-fig-1",
        name: "生命不减 1",
        exe: "FIG.EXE",
        locate: &[0x29, 0x54, 0x2D, 0xC7, 0x06],
        offset: 0,
        original: &[0x29, 0x54, 0x2D],
        patched: &[0x90, 0x90, 0x90],
    },
    PatchDef {
        id: "hp-fig-2",
        name: "生命不减 2",
        exe: "FIG.EXE",
        locate: &[0x29, 0x54, 0x2D, 0xE8, 0xE2],
        offset: 0,
        original: &[0x29, 0x54, 0x2D],
        patched: &[0x90, 0x90, 0x90],
    },
    PatchDef {
        id: "mp-fig-1",
        name: "仙术·内力不减 1",
        exe: "FIG.EXE",
        locate: &[0x29, 0x05, 0xC3, 0x8B, 0x44],
        offset: 0,
        original: &[0x29, 0x05],
        patched: &[0x90, 0x90],
    },
    PatchDef {
        id: "mp-fig-2",
        name: "仙术·内力不减 2",
        exe: "FIG.EXE",
        locate: &[0x29, 0x44, 0x35, 0xC3, 0x83],
        offset: 0,
        original: &[0x29, 0x44, 0x35],
        patched: &[0x90, 0x90, 0x90],
    },
    // ---- RPG.EXE ----
    PatchDef {
        id: "hp-rpg",
        name: "生命不减",
        exe: "RPG.EXE",
        locate: &[0x29, 0x7C, 0x2D, 0xB8, 0x00],
        offset: 0,
        original: &[0x29, 0x7C, 0x2D],
        patched: &[0x90, 0x90, 0x90],
    },
    PatchDef {
        id: "money-rpg-1",
        name: "金钱不减 1",
        exe: "RPG.EXE",
        locate: &[0x29, 0x16, 0x1B, 0x01, 0x3B],
        offset: 0,
        original: &[0x29, 0x16, 0x1B, 0x01],
        patched: &[0x90, 0x90, 0x90, 0x90],
    },
    PatchDef {
        id: "money-rpg-2",
        name: "金钱不减 2",
        exe: "RPG.EXE",
        locate: &[0x29, 0x06, 0x1B, 0x01, 0xA1],
        offset: 0,
        original: &[0x29, 0x06, 0x1B, 0x01],
        patched: &[0x90, 0x90, 0x90, 0x90],
    },
    PatchDef {
        id: "mp-rpg",
        name: "仙术·内力不减",
        exe: "RPG.EXE",
        locate: &[0x29, 0x05, 0x80, 0x3E, 0xA6],
        offset: 0,
        original: &[0x29, 0x05],
        patched: &[0x90, 0x90],
    },
];

/// 在 data 中搜索 needle，返回第一个匹配的位置
fn search_bytes(data: &[u8], needle: &[u8]) -> Option<usize> {
    data.windows(needle.len()).position(|w| w == needle)
}

/// 构造关闭补丁时的搜索串（locate 中 original 被替换为 patched 后的结果）
fn make_patched_locate(p: &PatchDef) -> Vec<u8> {
    let mut v = p.locate.to_vec();
    let end = p.offset + p.original.len();
    v[p.offset..end].copy_from_slice(p.patched);
    v
}

/// 应用补丁（开启）
fn apply_patch(exe_path: &str, p: &PatchDef) -> Result<(), String> {
    let mut data = fs::read(exe_path).map_err(|e| format!("读取 {} 失败: {}", exe_path, e))?;

    // 先搜索原始特征串（未补丁状态）
    let patched_locate = make_patched_locate(p);

    let (pos, found_patched) = if let Some(pos) = search_bytes(&data, p.locate) {
        (pos, false)
    } else if let Some(pos) = search_bytes(&data, &patched_locate) {
        (pos, true)
    } else {
        return Err(format!(
            "在 {} 中未找到特征串，补丁 \"{}\" 不适用于此版本游戏",
            exe_path, p.id
        ));
    };

    let replace_start = pos + p.offset;
    let replace_end = replace_start + p.original.len();
    let current = &data[replace_start..replace_end];

    if found_patched {
        return Err(format!("补丁 \"{}\" 已处于开启状态", p.id));
    }
    if current != p.original {
        return Err(format!(
            "偏移 0x{:X} 处数据异常（期望 {:02X?}，实际 {:02X?}），游戏版本不匹配",
            replace_start, p.original, current
        ));
    }

    data[replace_start..replace_end].copy_from_slice(p.patched);
    fs::write(exe_path, &data).map_err(|e| format!("写入 {} 失败: {}", exe_path, e))?;
    Ok(())
}

/// 恢复补丁（关闭）
fn disable_patch(exe_path: &str, p: &PatchDef) -> Result<(), String> {
    let mut data = fs::read(exe_path).map_err(|e| format!("读取 {} 失败: {}", exe_path, e))?;

    let patched_locate = make_patched_locate(p);

    let (pos, found_original) = if let Some(pos) = search_bytes(&data, &patched_locate) {
        (pos, false)
    } else if let Some(pos) = search_bytes(&data, p.locate) {
        (pos, true)
    } else {
        return Err(format!(
            "在 {} 中未找到特征串，补丁 \"{}\" 不适用于此版本游戏",
            exe_path, p.id
        ));
    };

    let replace_start = pos + p.offset;
    let replace_end = replace_start + p.patched.len();
    let current = &data[replace_start..replace_end];

    if found_original {
        return Err(format!("补丁 \"{}\" 已处于关闭状态", p.id));
    }
    if current != p.patched {
        return Err(format!(
            "偏移 0x{:X} 处数据异常（期望 {:02X?}，实际 {:02X?}），无法恢复",
            replace_start, p.patched, current
        ));
    }

    data[replace_start..replace_end].copy_from_slice(p.original);
    fs::write(exe_path, &data).map_err(|e| format!("写入 {} 失败: {}", exe_path, e))?;
    Ok(())
}

/// 列出所有补丁
fn list_patches() {
    println!("可用补丁：");
    println!();
    let mut last_exe = "";
    for p in PATCHES {
        if p.exe != last_exe {
            println!("  [{}]", p.exe);
            last_exe = p.exe;
        }
        println!(
            "    {:<16}  {}  (替换 {} 字节为 NOP)",
            p.id,
            p.name,
            p.original.len()
        );
    }
    println!();
    println!("用法:");
    println!("  swda_money_editor.exe patch <EXE路径> <补丁名> on");
    println!("  swda_money_editor.exe patch <EXE路径> <补丁名> off");
    println!("  swda_money_editor.exe patch <EXE路径> all on");
    println!("  swda_money_editor.exe patch <EXE路径> all off");
}

/// 在特定 exe 上查找属于它的补丁
fn patches_for_exe<'a>(exe_name: &str) -> impl Iterator<Item = &'a PatchDef> {
    let upper = exe_name.to_uppercase();
    PATCHES.iter().filter(move |p| {
        p.exe == upper || p.exe == exe_name
    })
}

// ============================================================
// 入口
// ============================================================

fn print_save_usage() {
    eprintln!("用法: swda_money_editor.exe <存档编号> [金额]");
    eprintln!("      存档编号: 1, 2, 3, 4, 5, Q");
    eprintln!("      金额范围: 0 ~ 65535（不填则只显示当前金钱）");
    eprintln!();
    eprintln!("      或: swda_money_editor.exe patch <EXE路径> <补丁名> on/off");
    eprintln!("          swda_money_editor.exe patch list");
}

fn cmd_save(args: &[String]) {
    if args.len() < 1 || args.len() > 2 {
        eprintln!("错误: 参数数量不正确");
        eprintln!();
        print_save_usage();
        process::exit(1);
    }

    let filename = match slot_to_filename(&args[0]) {
        Ok(f) => f,
        Err(msg) => {
            eprintln!("错误: {}", msg);
            process::exit(1);
        }
    };

    if !Path::new(&filename).exists() {
        eprintln!("错误: 存档不存在: {}", filename);
        process::exit(1);
    }

    let current = match read_current_money(&filename) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("错误: 无法读取存档 {}: {}", filename, e);
            process::exit(1);
        }
    };

    if args.len() == 1 {
        println!("存档 {}: {} 文钱", filename, current);
        return;
    }

    // 修改
    let money: u32 = match args[1].parse() {
        Ok(m) => m,
        Err(_) => {
            eprintln!("错误: 金额 \"{}\" 不是有效的整数", args[1]);
            process::exit(1);
        }
    };

    if money > MAX_MONEY {
        eprintln!("错误: 金额 {} 超出上限 {}", money, MAX_MONEY);
        process::exit(1);
    }

    println!("存档 {}: {} 文钱 → {} 文钱", filename, current, money);

    if let Err(e) = backup_file(&filename) {
        eprintln!("备份失败: {}", e);
        process::exit(1);
    }

    if let Err(e) = modify_money(&filename, money) {
        eprintln!("修改失败: {}", e);
        process::exit(1);
    }

    match verify_money(&filename, money) {
        Ok(true) => println!("  [OK] 修改成功"),
        Ok(false) => {
            eprintln!("  [FAIL] 验证失败");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("验证出错: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_patch(args: &[String]) {
    // patch list
    if args.len() == 1 && args[0] == "list" {
        list_patches();
        return;
    }

    if args.len() != 3 {
        eprintln!("错误: patch 参数不正确");
        eprintln!("用法:");
        eprintln!("  swda_money_editor.exe patch list");
        eprintln!("  swda_money_editor.exe patch <EXE路径> <补丁名> on");
        eprintln!("  swda_money_editor.exe patch <EXE路径> <补丁名> off");
        process::exit(1);
    }

    let exe_path = &args[0];
    let patch_id = &args[1];
    let action = &args[2];

    // 检查 exe 是否存在
    if !Path::new(exe_path).exists() {
        eprintln!("错误: 文件不存在: {}", exe_path);
        process::exit(1);
    }

    // 读取 exe 文件名用于匹配补丁
    let exe_name = Path::new(exe_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let apply_all = patch_id == "all";

    // 筛选要操作的补丁
    let targets: Vec<&PatchDef> = if apply_all {
        patches_for_exe(exe_name).collect()
    } else if let Some(p) = PATCHES.iter().find(|p| p.id == patch_id) {
        // 验证补丁对应的 exe 是否匹配
        let exe_upper = exe_name.to_uppercase();
        if p.exe != exe_upper {
            eprintln!(
                "错误: 补丁 \"{}\" 属于 {}，但指定文件是 {}",
                p.id, p.exe, exe_name
            );
            process::exit(1);
        }
        vec![p]
    } else {
        eprintln!("错误: 未知补丁 \"{}\"（用 list 查看可用补丁）", patch_id);
        process::exit(1);
    };

    if targets.is_empty() {
        eprintln!("{} 没有可用的补丁", exe_path);
        process::exit(1);
    }

    // 执行
    let do_enable = match action.as_str() {
        "on" => true,
        "off" => false,
        _ => {
            eprintln!("错误: 动作必须是 on 或 off");
            process::exit(1);
        }
    };

    // 先备份 exe（只备一次）
    if do_enable {
        if let Err(e) = backup_file(exe_path) {
            eprintln!("备份失败: {}", e);
            process::exit(1);
        }
    }

    let mut fail = 0;

    for p in &targets {
        print!("  {} ({}) ... ", p.name, if do_enable { "开启" } else { "关闭" });
        let result = if do_enable {
            apply_patch(exe_path, p)
        } else {
            disable_patch(exe_path, p)
        };
        match result {
            Ok(()) => {
                println!("[OK]");
            }
            Err(msg) => {
                println!("[FAIL] {}", msg);
                fail += 1;
            }
        }
    }

    if fail > 0 {
        process::exit(1);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_save_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "patch" => cmd_patch(&args[2..]),
        _ => cmd_save(&args[1..]),
    }
}

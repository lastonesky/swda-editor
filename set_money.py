#!/usr/bin/env python3
"""
轩辕剑外传枫之舞 · 存档金钱修改器 & EXE 补丁工具

用法:
  set_money.py <存档编号> [金额]         查看/修改存档金钱
  set_money.py patch list                列出可用补丁
  set_money.py patch <EXE路径> <补丁名> on/off   开启/关闭补丁
  set_money.py patch <EXE路径> all on/off        全部开启/关闭

存档编号: 1, 2, 3, 4, 5, Q
金额范围: 0 ~ 65535（不填则只显示当前金钱）
"""

import sys
import os
import struct

# ============================================================
# 存档金钱修改
# ============================================================

MONEY_OFFSET = 0x011B
MONEY_SIZE = 2
MIN_FILE_SIZE = MONEY_OFFSET + MONEY_SIZE
MAX_MONEY = 65535

SAVE_SLOTS = {"1": "SAVE.ZA1", "2": "SAVE.ZA2", "3": "SAVE.ZA3",
              "4": "SAVE.ZA4", "5": "SAVE.ZA5", "Q": "SAVE.ZAQ"}


def slot_to_filename(slot: str) -> str:
    upper = slot.upper()
    if upper in SAVE_SLOTS:
        return SAVE_SLOTS[upper]
    raise ValueError(f"无效的存档编号: \"{slot}\"（可用: 1,2,3,4,5,Q）")


def backup_file(path: str):
    bak_path = path + ".bak"
    if os.path.exists(bak_path):
        print(f"  备份已存在: {bak_path}")
        return
    import shutil
    shutil.copy2(path, bak_path)
    print(f"  已备份: {path} → {bak_path}")


def read_money(data: bytes) -> int:
    lo = data[MONEY_OFFSET]
    hi = data[MONEY_OFFSET + 1]
    return lo | (hi << 8)


def read_current_money(path: str) -> int:
    with open(path, "rb") as f:
        data = f.read()
    if len(data) < MIN_FILE_SIZE:
        raise ValueError("文件太小")
    return read_money(data)


def modify_save_money(path: str, money: int):
    with open(path, "rb") as f:
        data = bytearray(f.read())
    if len(data) < MIN_FILE_SIZE:
        print(f"错误: 文件太小 ({len(data)} 字节)")
        sys.exit(1)
    old_money = read_money(data)
    struct.pack_into("<H", data, MONEY_OFFSET, money)
    with open(path, "wb") as f:
        f.write(data)
    return old_money


def verify_save_money(path: str, expected: int) -> bool:
    try:
        return read_current_money(path) == expected
    except Exception:
        return False


# ============================================================
# EXE 补丁
# ============================================================

PATCHES = [
    # (id, name, exe, locate_bytes, offset, original_bytes, patched_bytes)
    # ---- FIG.EXE ----
    ("fast-level",  "快速升级",           "FIG.EXE",
     bytes([0x29, 0x44, 0x39, 0xA1, 0xAC]), 0,
     bytes([0x29, 0x44, 0x39]), bytes([0x90, 0x90, 0x90])),

    ("hp-fig-1",    "生命不减 1",          "FIG.EXE",
     bytes([0x29, 0x54, 0x2D, 0xC7, 0x06]), 0,
     bytes([0x29, 0x54, 0x2D]), bytes([0x90, 0x90, 0x90])),

    ("hp-fig-2",    "生命不减 2",          "FIG.EXE",
     bytes([0x29, 0x54, 0x2D, 0xE8, 0xE2]), 0,
     bytes([0x29, 0x54, 0x2D]), bytes([0x90, 0x90, 0x90])),

    ("mp-fig-1",    "仙术·内力不减 1",    "FIG.EXE",
     bytes([0x29, 0x05, 0xC3, 0x8B, 0x44]), 0,
     bytes([0x29, 0x05]), bytes([0x90, 0x90])),

    ("mp-fig-2",    "仙术·内力不减 2",    "FIG.EXE",
     bytes([0x29, 0x44, 0x35, 0xC3, 0x83]), 0,
     bytes([0x29, 0x44, 0x35]), bytes([0x90, 0x90, 0x90])),

    # ---- RPG.EXE ----
    ("hp-rpg",      "生命不减",           "RPG.EXE",
     bytes([0x29, 0x7C, 0x2D, 0xB8, 0x00]), 0,
     bytes([0x29, 0x7C, 0x2D]), bytes([0x90, 0x90, 0x90])),

    ("money-rpg-1", "金钱不减 1",          "RPG.EXE",
     bytes([0x29, 0x16, 0x1B, 0x01, 0x3B]), 0,
     bytes([0x29, 0x16, 0x1B, 0x01]), bytes([0x90, 0x90, 0x90, 0x90])),

    ("money-rpg-2", "金钱不减 2",          "RPG.EXE",
     bytes([0x29, 0x06, 0x1B, 0x01, 0xA1]), 0,
     bytes([0x29, 0x06, 0x1B, 0x01]), bytes([0x90, 0x90, 0x90, 0x90])),

    ("mp-rpg",      "仙术·内力不减",      "RPG.EXE",
     bytes([0x29, 0x05, 0x80, 0x3E, 0xA6]), 0,
     bytes([0x29, 0x05]), bytes([0x90, 0x90])),
]


def search_bytes(data: bytes, needle: bytes) -> int | None:
    pos = data.find(needle)
    return pos if pos >= 0 else None


def make_patched_locate(locate: bytes, offset: int, original: bytes, patched: bytes) -> bytes:
    lst = list(locate)
    lst[offset:offset + len(original)] = patched
    return bytes(lst)


def apply_patch(exe_path: str, pid: str, name: str,
                locate: bytes, offset: int, original: bytes, patched: bytes) -> str | None:
    """开启补丁，成功返回 None，失败返回错误信息"""
    try:
        with open(exe_path, "rb") as f:
            data = bytearray(f.read())
    except Exception as e:
        return f"读取失败: {e}"

    patched_locate = make_patched_locate(locate, offset, original, patched)

    # 先搜原始特征串
    pos = search_bytes(data, locate)
    found_patched = False
    if pos is None:
        # 再搜已补丁特征串
        pos = search_bytes(data, patched_locate)
        if pos is not None:
            found_patched = True
        else:
            return f"未找到特征串，补丁 \"{pid}\" 不适用于此版本游戏"

    start = pos + offset
    end = start + len(original)
    current = bytes(data[start:end])

    if found_patched:
        return f"补丁 \"{pid}\" 已处于开启状态"
    if current != original:
        return f"偏移 0x{start:X} 处数据异常，游戏版本不匹配"

    data[start:end] = patched
    try:
        with open(exe_path, "wb") as f:
            f.write(data)
    except Exception as e:
        return f"写入失败: {e}"

    return None


def disable_patch(exe_path: str, pid: str, name: str,
                  locate: bytes, offset: int, original: bytes, patched: bytes) -> str | None:
    """关闭补丁，成功返回 None，失败返回错误信息"""
    try:
        with open(exe_path, "rb") as f:
            data = bytearray(f.read())
    except Exception as e:
        return f"读取失败: {e}"

    patched_locate = make_patched_locate(locate, offset, original, patched)

    # 先搜已补丁特征串
    pos = search_bytes(data, patched_locate)
    found_original = False
    if pos is None:
        # 再搜原始特征串
        pos = search_bytes(data, locate)
        if pos is not None:
            found_original = True
        else:
            return f"未找到特征串，补丁 \"{pid}\" 不适用于此版本游戏"

    start = pos + offset
    end = start + len(patched)
    current = bytes(data[start:end])

    if found_original:
        return f"补丁 \"{pid}\" 已处于关闭状态"
    if current != patched:
        return f"偏移 0x{start:X} 处数据异常，无法恢复"

    data[start:end] = original
    try:
        with open(exe_path, "wb") as f:
            f.write(data)
    except Exception as e:
        return f"写入失败: {e}"

    return None


def list_patches():
    print("可用补丁：\n")
    last_exe = ""
    for pid, name, exe, *_ in PATCHES:
        if exe != last_exe:
            print(f"  [{exe}]")
            last_exe = exe
        print(f"    {pid:<16}  {name}")
    print()
    print("用法:")
    print("  set_money.py patch <EXE路径> <补丁名> on")
    print("  set_money.py patch <EXE路径> <补丁名> off")
    print("  set_money.py patch <EXE路径> all on")
    print("  set_money.py patch <EXE路径> all off")


# ============================================================
# 命令分发
# ============================================================

def print_usage():
    print("用法:")
    print("  set_money.py <存档编号> [金额]         查看/修改存档金钱")
    print("  set_money.py patch list                列出可用补丁")
    print("  set_money.py patch <EXE路径> <补丁名> on/off   开启/关闭补丁")
    print("  set_money.py patch <EXE路径> all on/off        全部开启/关闭")
    print()
    print("存档编号: 1, 2, 3, 4, 5, Q")
    print("金额范围: 0 ~ 65535（不填则只显示当前金钱）")


def cmd_save(args: list[str]):
    if len(args) < 1 or len(args) > 2:
        print("错误: 参数数量不正确\n")
        print_usage()
        sys.exit(1)

    try:
        filename = slot_to_filename(args[0])
    except ValueError as e:
        print(f"错误: {e}")
        sys.exit(1)

    if not os.path.isfile(filename):
        print(f"错误: 存档不存在: {filename}")
        sys.exit(1)

    try:
        current = read_current_money(filename)
    except Exception as e:
        print(f"错误: 无法读取存档 {filename}: {e}")
        sys.exit(1)

    if len(args) == 1:
        print(f"存档 {filename}: {current} 文钱")
        return

    # 修改
    money_str = args[1]
    if not money_str.isdigit():
        print(f"错误: 金额 \"{money_str}\" 不是有效的整数")
        sys.exit(1)

    money = int(money_str)
    if money > MAX_MONEY:
        print(f"错误: 金额 {money} 超出上限 {MAX_MONEY}")
        sys.exit(1)

    print(f"存档 {filename}: {current} 文钱 → {money} 文钱")
    backup_file(filename)
    old = modify_save_money(filename, money)

    if verify_save_money(filename, money):
        print("  [OK] 修改成功")
    else:
        print("  [FAIL] 验证失败")
        sys.exit(1)


def cmd_patch(args: list[str]):
    # patch list
    if len(args) == 1 and args[0] == "list":
        list_patches()
        return

    if len(args) != 3:
        print("错误: patch 参数不正确\n")
        print("用法:")
        print("  set_money.py patch list")
        print("  set_money.py patch <EXE路径> <补丁名> on")
        print("  set_money.py patch <EXE路径> <补丁名> off")
        sys.exit(1)

    exe_path = args[0]
    patch_id = args[1]
    action = args[2]

    if not os.path.isfile(exe_path):
        print(f"错误: 文件不存在: {exe_path}")
        sys.exit(1)

    do_enable = {"on": True, "off": False}.get(action)
    if do_enable is None:
        print("错误: 动作必须是 on 或 off")
        sys.exit(1)

    exe_name = os.path.basename(exe_path).upper()
    apply_all = patch_id == "all"

    # 筛选补丁
    targets = []
    for p in PATCHES:
        pid, name, p_exe, locate, offset, original, patched = p
        if apply_all:
            if p_exe.upper() == exe_name:
                targets.append(p)
        elif pid == patch_id:
            if p_exe.upper() != exe_name:
                print(f"错误: 补丁 \"{pid}\" 属于 {p_exe}，但指定文件是 {exe_name}")
                sys.exit(1)
            targets.append(p)
            break
    else:
        if not apply_all:
            print(f"错误: 未知补丁 \"{patch_id}\"（用 list 查看可用补丁）")
            sys.exit(1)

    if not targets:
        print(f"{exe_path} 没有可用的补丁")
        sys.exit(1)

    # 备份（只备一次）
    if do_enable:
        backup_file(exe_path)

    fail = 0
    for p in targets:
        pid, name, _exe, locate, offset, original, patched = p
        status = "开启" if do_enable else "关闭"
        print(f"  {name} ({status}) ... ", end="", flush=True)

        err = apply_patch(exe_path, pid, name, locate, offset, original, patched) if do_enable else \
              disable_patch(exe_path, pid, name, locate, offset, original, patched)
        if err is None:
            print("[OK]")
        else:
            print(f"[FAIL] {err}")
            fail += 1

    if fail > 0:
        sys.exit(1)


def main():
    if len(sys.argv) < 2:
        print_usage()
        sys.exit(1)

    if sys.argv[1] == "patch":
        cmd_patch(sys.argv[2:])
    else:
        cmd_save(sys.argv[1:])


if __name__ == "__main__":
    main()

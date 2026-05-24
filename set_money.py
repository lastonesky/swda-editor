#!/usr/bin/env python3
"""
轩辕剑外传枫之舞 - 存档金钱修改器
用法: set_money.py <存档文件路径> <金额>
      金额范围: 0 ~ 65535
      自动备份原文件为 <路径>.bak
"""

import sys
import os
import struct

MONEY_OFFSET = 0x011B
MONEY_SIZE = 2  # 16-bit little-endian
MIN_FILE_SIZE = MONEY_OFFSET + MONEY_SIZE
MAX_MONEY = 65535  # u16 max


def print_usage():
    print("用法: set_money.py <存档文件路径> <金额>")
    print("说明: 修改轩辕剑外传枫之舞存档的金钱数量")
    print("      金额范围: 0 ~ 65535")
    print("      会自动备份原文件为 <路径>.bak")


def backup_file(path: str):
    bak_path = path + ".bak"
    if os.path.exists(bak_path):
        print(f"  备份已存在: {bak_path}")
        return
    import shutil
    shutil.copy2(path, bak_path)
    print(f"  已备份: {path} → {bak_path}")


def read_money(data: bytes) -> int:
    """从数据中读取当前金钱（16位小端）"""
    lo = data[MONEY_OFFSET]
    hi = data[MONEY_OFFSET + 1]
    return lo | (hi << 8)


def modify_money(path: str, money: int):
    """修改存档文件中的金钱"""
    with open(path, "rb") as f:
        data = bytearray(f.read())

    if len(data) < MIN_FILE_SIZE:
        print(f"错误: 文件太小 ({len(data)} 字节)，需要至少 {MIN_FILE_SIZE} 字节")
        sys.exit(1)

    old_money = read_money(data)
    struct.pack_into("<H", data, MONEY_OFFSET, money)

    with open(path, "wb") as f:
        f.write(data)

    print(f"  金钱: {old_money} → {money} 文钱")


def verify(path: str, expected: int) -> bool:
    """验证修改结果"""
    with open(path, "rb") as f:
        data = f.read()
    if len(data) < MIN_FILE_SIZE:
        return False
    return read_money(data) == expected


def main():
    if len(sys.argv) != 3:
        print("错误: 参数数量不正确")
        print()
        print_usage()
        sys.exit(1)

    path = sys.argv[1]
    money_str = sys.argv[2]

    if not path.strip():
        print("错误: 存档路径不能为空")
        sys.exit(1)

    if not money_str.isdigit():
        print(f"错误: 金额 \"{money_str}\" 不是有效的整数")
        sys.exit(1)

    money = int(money_str)
    if money > MAX_MONEY:
        print(f"错误: 金额 {money} 超出上限 {MAX_MONEY}（16位无符号最大值）")
        sys.exit(1)

    if not os.path.isfile(path):
        print(f"错误: 文件不存在: {path}")
        sys.exit(1)

    print(f"处理: {path}")
    print(f"目标: {money} 文钱")

    # 1. 备份
    backup_file(path)

    # 2. 修改
    try:
        modify_money(path, money)
    except Exception as e:
        print(f"修改失败: {e}")
        sys.exit(1)

    # 3. 验证
    try:
        if verify(path, money):
            print(f"[OK] 修改成功，已写入 {money} 文钱")
        else:
            print("[FAIL] 验证失败，写入结果与预期不符")
            sys.exit(1)
    except Exception as e:
        print(f"验证过程出错: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()

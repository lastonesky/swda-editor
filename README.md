# 轩辕剑外传枫之舞 · 存档金钱修改器

`swda-money-editor` 是一个命令行工具，用于查看和修改 DOS 经典游戏《轩辕剑外传枫之舞》的存档金钱。

纯 Rust 实现，无外部依赖，编译后单文件 170KB。

## 功能

- 查看任意存档位的当前金钱
- 修改指定存档位的金钱（自动备份原文件）
- 写入后自动回读验证

## 环境要求

- **编译**: Rust 1.85+（edition 2024）
- **运行**: Windows x64（Windows 7 SP1 需安装 [VC++ Redistributable 2015+](https://aka.ms/vs/17/release/vc_redist.x64.exe)）

## 编译

```bash
git clone https://github.com/你的用户名/swda-money-editor.git
cd swda-money-editor
cargo build --release
```

编译产物在 `target/release/swda_money_editor.exe`。

## 用法

将 `swda_money_editor.exe` 放到《枫之舞》的 `SAVE` 目录下，在命令行中运行。

### 查看金钱

```bash
swda_money_editor.exe 1
# 输出: 存档 SAVE.ZA1: 50 文钱
```

### 修改金钱

```bash
swda_money_editor.exe 2 9999
# 输出:
#   存档 SAVE.ZA2: 40 文钱 → 9999 文钱
#   已备份: SAVE.ZA2 → SAVE.ZA2.bak
#   [OK] 修改成功
```

### 存档编号

| 编号 | 对应文件 |
|------|----------|
| 1 | SAVE.ZA1 |
| 2 | SAVE.ZA2 |
| 3 | SAVE.ZA3 |
| 4 | SAVE.ZA4 |
| 5 | SAVE.ZA5 |
| Q | SAVE.ZAQ |

## 技术细节

金钱数据在存档文件中的存储位置：

- **偏移**: `0x011B`（十进制 283）
- **格式**: 16 位无符号整数，小端序（`uint16 LE`）
- **上限**: 65535 文钱

例如 50 文钱在二进制中表示为 `0x32 0x00`，9999 文钱表示为 `0x0F 0x27`。

## 许可证

MIT

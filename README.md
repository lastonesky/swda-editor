# 轩辕剑外传枫之舞 · 存档金钱修改器 & EXE 补丁工具

`swda-money-editor` 是一个命令行工具，集成了两项功能：

- **存档编辑**：查看和修改《轩辕剑外传枫之舞》的存档金钱
- **EXE 补丁**：对游戏主程序（FIG.EXE / RPG.EXE）打补丁，实现快速升级、生命不减等效果，且可随时开关

纯 Rust 实现，无外部依赖，编译后单文件约 180KB。

## 功能

### 存档金钱
- 查看任意存档位的当前金钱
- 修改指定存档位的金钱（自动备份原文件）
- 写入后自动回读验证

### EXE 补丁

| 补丁 ID | 说明 | 目标文件 |
|---------|------|----------|
| `fast-level` | 快速升级 | FIG.EXE |
| `hp-fig-1` | 生命不减 1 | FIG.EXE |
| `hp-fig-2` | 生命不减 2 | FIG.EXE |
| `mp-fig-1` | 仙术·内力不减 1 | FIG.EXE |
| `mp-fig-2` | 仙术·内力不减 2 | FIG.EXE |
| `hp-rpg` | 生命不减 | RPG.EXE |
| `money-rpg-1` | 金钱不减 1 | RPG.EXE |
| `money-rpg-2` | 金钱不减 2 | RPG.EXE |
| `mp-rpg` | 仙术·内力不减 | RPG.EXE |

所有补丁均可随时开启和关闭，开启时自动备份原文件。

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

### 存档金钱

将 `swda_money_editor.exe` 放到《枫之舞》的 `SAVE` 目录下运行。

**查看金钱：**
```
swda_money_editor.exe 1
→ 存档 SAVE.ZA1: 50 文钱
```

**修改金钱：**
```
swda_money_editor.exe 2 9999
→ 存档 SAVE.ZA2: 40 文钱 → 9999 文钱
  已备份: SAVE.ZA2 → SAVE.ZA2.bak
  [OK] 修改成功
```

**存档编号对照：**

| 编号 | 对应文件 |
|------|----------|
| 1 | SAVE.ZA1 |
| 2 | SAVE.ZA2 |
| 3 | SAVE.ZA3 |
| 4 | SAVE.ZA4 |
| 5 | SAVE.ZA5 |
| Q | SAVE.ZAQ |

### EXE 补丁

在游戏根目录（FIG.EXE / RPG.EXE 所在目录）运行。

**列出所有可用补丁：**
```
swda_money_editor.exe patch list
```

**开启单个补丁：**
```
swda_money_editor.exe patch FIG.EXE fast-level on
→ 已备份: FIG.EXE → FIG.EXE.bak
  快速升级 (开启) ... [OK]
```

**关闭单个补丁：**
```
swda_money_editor.exe patch FIG.EXE fast-level off
→ 快速升级 (关闭) ... [OK]
```

**一次开启/关闭某个 EXE 的所有补丁：**
```
swda_money_editor.exe patch RPG.EXE all on
swda_money_editor.exe patch FIG.EXE all off
```

补丁可重复开关，已开启时再次开启会提示"已处于开启状态"，不会重复修改。

## 技术细节

### 金钱数据

- **偏移**: `0x011B`（十进制 283）
- **格式**: 16 位无符号整数，小端序（`uint16 LE`）
- **上限**: 65535 文钱

例如 50 文钱在二进制中表示为 `0x32 0x00`，9999 文钱表示为 `0x0F 0x27`。

### 补丁原理

补丁通过将游戏中的减法指令（`SUB`，操作码 `0x29`）替换为空操作（`NOP`，操作码 `0x90`），阻止数值减少。因为修改的是内存中的指令而非存档数据，所以需要直接修改 EXE 文件。

特征串搜索确保了定位的准确性，每个补丁使用 5 字节特征串进行唯一匹配。

## 许可证

MIT

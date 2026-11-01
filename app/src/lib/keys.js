// 完整 104 键 ANSI 布局，24 列 × 6 行。
// 主键区 0-14 列，导航区 16-18 列，小键盘 19-22 列。
export const FULL_LAYOUT = [
  { label: "Esc", vk: 27, col: 0, row: 0, w: 1 },
  { label: "F1", vk: 112, col: 2, row: 0, w: 1 },
  { label: "F2", vk: 113, col: 3, row: 0, w: 1 },
  { label: "F3", vk: 114, col: 4, row: 0, w: 1 },
  { label: "F4", vk: 115, col: 5, row: 0, w: 1 },
  { label: "F5", vk: 116, col: 6.5, row: 0, w: 1 },
  { label: "F6", vk: 117, col: 7.5, row: 0, w: 1 },
  { label: "F7", vk: 118, col: 8.5, row: 0, w: 1 },
  { label: "F8", vk: 119, col: 9.5, row: 0, w: 1 },
  { label: "F9", vk: 120, col: 11, row: 0, w: 1 },
  { label: "F10", vk: 121, col: 12, row: 0, w: 1 },
  { label: "F11", vk: 122, col: 13, row: 0, w: 1 },
  { label: "F12", vk: 123, col: 14, row: 0, w: 1 },
  { label: "PrtSc", vk: 44, col: 15.5, row: 0, w: 1 },
  { label: "ScrLk", vk: 145, col: 16.5, row: 0, w: 1 },
  { label: "Pause", vk: 19, col: 17.5, row: 0, w: 1 },

  { label: "`", vk: 192, col: 0, row: 1, w: 1 },
  { label: "1", vk: 49, col: 1, row: 1, w: 1 },
  { label: "2", vk: 50, col: 2, row: 1, w: 1 },
  { label: "3", vk: 51, col: 3, row: 1, w: 1 },
  { label: "4", vk: 52, col: 4, row: 1, w: 1 },
  { label: "5", vk: 53, col: 5, row: 1, w: 1 },
  { label: "6", vk: 54, col: 6, row: 1, w: 1 },
  { label: "7", vk: 55, col: 7, row: 1, w: 1 },
  { label: "8", vk: 56, col: 8, row: 1, w: 1 },
  { label: "9", vk: 57, col: 9, row: 1, w: 1 },
  { label: "0", vk: 48, col: 10, row: 1, w: 1 },
  { label: "-", vk: 189, col: 11, row: 1, w: 1 },
  { label: "=", vk: 187, col: 12, row: 1, w: 1 },
  { label: "Bksp", vk: 8, col: 13, row: 1, w: 2 },
  { label: "Ins", vk: 45, col: 15.5, row: 1, w: 1 },
  { label: "Home", vk: 36, col: 16.5, row: 1, w: 1 },
  { label: "PgUp", vk: 33, col: 17.5, row: 1, w: 1 },
  { label: "Num", vk: 144, col: 19, row: 1, w: 1 },
  { label: "/", vk: 111, col: 20, row: 1, w: 1 },
  { label: "*", vk: 106, col: 21, row: 1, w: 1 },
  { label: "-", vk: 109, col: 22, row: 1, w: 1 },

  { label: "Tab", vk: 9, col: 0, row: 2, w: 1.5 },
  { label: "Q", vk: 81, col: 1.5, row: 2, w: 1 },
  { label: "W", vk: 87, col: 2.5, row: 2, w: 1 },
  { label: "E", vk: 69, col: 3.5, row: 2, w: 1 },
  { label: "R", vk: 82, col: 4.5, row: 2, w: 1 },
  { label: "T", vk: 84, col: 5.5, row: 2, w: 1 },
  { label: "Y", vk: 89, col: 6.5, row: 2, w: 1 },
  { label: "U", vk: 85, col: 7.5, row: 2, w: 1 },
  { label: "I", vk: 73, col: 8.5, row: 2, w: 1 },
  { label: "O", vk: 79, col: 9.5, row: 2, w: 1 },
  { label: "P", vk: 80, col: 10.5, row: 2, w: 1 },
  { label: "[", vk: 219, col: 11.5, row: 2, w: 1 },
  { label: "]", vk: 221, col: 12.5, row: 2, w: 1 },
  { label: "\\", vk: 220, col: 13.5, row: 2, w: 1.5 },
  { label: "Del", vk: 46, col: 15.5, row: 2, w: 1 },
  { label: "End", vk: 35, col: 16.5, row: 2, w: 1 },
  { label: "PgDn", vk: 34, col: 17.5, row: 2, w: 1 },
  { label: "7", vk: 103, col: 19, row: 2, w: 1 },
  { label: "8", vk: 104, col: 20, row: 2, w: 1 },
  { label: "9", vk: 105, col: 21, row: 2, w: 1 },
  { label: "+", vk: 107, col: 22, row: 2, w: 1, h: 2 },

  { label: "Caps", vk: 20, col: 0, row: 3, w: 1.75 },
  { label: "A", vk: 65, col: 1.75, row: 3, w: 1 },
  { label: "S", vk: 83, col: 2.75, row: 3, w: 1 },
  { label: "D", vk: 68, col: 3.75, row: 3, w: 1 },
  { label: "F", vk: 70, col: 4.75, row: 3, w: 1 },
  { label: "G", vk: 71, col: 5.75, row: 3, w: 1 },
  { label: "H", vk: 72, col: 6.75, row: 3, w: 1 },
  { label: "J", vk: 74, col: 7.75, row: 3, w: 1 },
  { label: "K", vk: 75, col: 8.75, row: 3, w: 1 },
  { label: "L", vk: 76, col: 9.75, row: 3, w: 1 },
  { label: ";", vk: 186, col: 10.75, row: 3, w: 1 },
  { label: "'", vk: 222, col: 11.75, row: 3, w: 1 },
  { label: "Enter", vk: 13, col: 12.75, row: 3, w: 2.25 },
  { label: "↑", vk: 38, col: 16.5, row: 3, w: 1 },
  { label: "4", vk: 100, col: 19, row: 3, w: 1 },
  { label: "5", vk: 101, col: 20, row: 3, w: 1 },
  { label: "6", vk: 102, col: 21, row: 3, w: 1 },

  { label: "LShift", vk: 160, col: 0, row: 4, w: 2.25 },
  { label: "Z", vk: 90, col: 2.25, row: 4, w: 1 },
  { label: "X", vk: 88, col: 3.25, row: 4, w: 1 },
  { label: "C", vk: 67, col: 4.25, row: 4, w: 1 },
  { label: "V", vk: 86, col: 5.25, row: 4, w: 1 },
  { label: "B", vk: 66, col: 6.25, row: 4, w: 1 },
  { label: "N", vk: 78, col: 7.25, row: 4, w: 1 },
  { label: "M", vk: 77, col: 8.25, row: 4, w: 1 },
  { label: ",", vk: 188, col: 9.25, row: 4, w: 1 },
  { label: ".", vk: 190, col: 10.25, row: 4, w: 1 },
  { label: "/", vk: 191, col: 11.25, row: 4, w: 1 },
  { label: "RShift", vk: 161, col: 12.25, row: 4, w: 2.75 },
  { label: "←", vk: 37, col: 15.5, row: 4, w: 1 },
  { label: "↓", vk: 40, col: 16.5, row: 4, w: 1 },
  { label: "→", vk: 39, col: 17.5, row: 4, w: 1 },
  { label: "1", vk: 97, col: 19, row: 4, w: 1 },
  { label: "2", vk: 98, col: 20, row: 4, w: 1 },
  { label: "3", vk: 99, col: 21, row: 4, w: 1 },
  { label: "NumEnt", vk: 108, col: 22, row: 4, w: 1, h: 2 },

  { label: "LCtrl", vk: 162, col: 0, row: 5, w: 1.25 },
  { label: "LWin", vk: 91, col: 1.25, row: 5, w: 1.25 },
  { label: "LAlt", vk: 164, col: 2.5, row: 5, w: 1.25 },
  { label: "Space", vk: 32, col: 3.75, row: 5, w: 6.25 },
  { label: "RAlt", vk: 165, col: 10, row: 5, w: 1.25 },
  { label: "RWin", vk: 92, col: 11.25, row: 5, w: 1.25 },
  { label: "Menu", vk: 93, col: 12.5, row: 5, w: 1.25 },
  { label: "RCtrl", vk: 163, col: 13.75, row: 5, w: 1.25 },
  { label: "0", vk: 96, col: 19, row: 5, w: 2 },
  { label: ".", vk: 110, col: 21, row: 5, w: 1 }
];

export function handOf(vk) {
  const left = new Set([27, 192, 49, 50, 51, 52, 53, 54, 9, 81, 87, 69, 82, 84, 20, 65, 83, 68, 70, 71, 160, 90, 88, 67, 86, 66, 162, 91, 164, 112, 113, 114, 115, 116]);
  const right = new Set([
    8, 55, 56, 57, 48, 189, 187, 219, 221, 220, 89, 85, 73, 79, 80, 72, 74, 75, 76, 186, 222, 13, 78, 77, 188, 190, 191, 161, 163, 92, 93, 165,
    33, 34, 35, 36, 37, 38, 39, 40, 44, 45, 46, 19, 144, 145, 111, 106, 109, 103, 104, 105, 107, 100, 101, 102, 97, 98, 99, 108, 96, 110,
    117, 118, 119, 120, 121, 122, 123
  ]);
  if (left.has(vk)) return "left";
  if (right.has(vk)) return "right";
  return "center";
}

export function keyZone(vk) {
  if ((vk >= 65 && vk <= 90)) return "字母键";
  if ((vk >= 48 && vk <= 57) || (vk >= 96 && vk <= 105)) return "数字键";
  if ([8, 9, 13, 20, 27, 32, 46].includes(vk)) return "控制与编辑";
  if ([16, 17, 18, 91, 92, 93, 160, 161, 162, 163, 164, 165].includes(vk)) return "修饰键";
  if (vk >= 112 && vk <= 123) return "功能键 F1-F12";
  if (vk >= 33 && vk <= 45) return "导航与方向";
  return "符号与小键盘";
}

export function labelOf(vk, layout = FULL_LAYOUT) {
  const key = layout.find((k) => k.vk === vk);
  if (key) return key.label;
  const aliases = {
    8: "Bksp",
    9: "Tab",
    13: "Enter",
    16: "Shift",
    17: "Ctrl",
    18: "Alt",
    20: "Caps",
    27: "Esc",
    32: "Space",
    91: "LWin",
    92: "RWin",
    160: "LShift",
    161: "RShift",
    162: "LCtrl",
    163: "RCtrl",
    164: "LAlt",
    165: "RAlt"
  };
  if (aliases[vk]) return aliases[vk];
  if (vk >= 96 && vk <= 105) return `Num${vk - 96}`;
  return `VK_${vk}`;
}

export function shortcutLabel(mods, vk, layout = FULL_LAYOUT) {
  const parts = [];
  if (mods & 2) parts.push("Ctrl");
  if (mods & 1) parts.push("Shift");
  if (mods & 4) parts.push("Alt");
  if (mods & 8) parts.push("Win");
  parts.push(labelOf(vk, layout));
  return parts.join(" + ");
}

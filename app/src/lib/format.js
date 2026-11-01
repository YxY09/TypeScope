export function fmtNum(n) {
  if (n === null || n === undefined || Number.isNaN(Number(n))) return "0";
  return Number(n).toLocaleString("en-US");
}

export function fmtDur(ms) {
  if (!ms || ms <= 0) return "0 秒";
  const total = Math.round(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) return `${h} 小时 ${m} 分`;
  if (m > 0) return `${m} 分 ${s} 秒`;
  return `${s} 秒`;
}

export function fmtDurShort(ms) {
  if (!ms || ms <= 0) return "0m";
  const total = Math.round(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export function fmtDay(day) {
  if (!day) return "—";
  const s = String(day);
  if (s.length !== 8) return s;
  return `${s.slice(0, 4)}-${s.slice(4, 6)}-${s.slice(6, 8)}`;
}

export function fmtShort(day) {
  if (!day) return "—";
  const s = String(day);
  if (s.length !== 8) return s;
  return `${s.slice(4, 6)}-${s.slice(6, 8)}`;
}

export function fmtMinute(minute) {
  if (minute === null || minute === undefined || minute < 0) return "—";
  const h = Math.floor(minute / 60);
  const m = minute % 60;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
}

export function kpm(physical, activeMs) {
  if (!activeMs || activeMs <= 0 || !physical || physical <= 0) return null;
  return Math.round((physical * 60000) / activeMs);
}

export function weekdayMon(day) {
  const s = String(day);
  const date = new Date(Number(s.slice(0, 4)), Number(s.slice(4, 6)) - 1, Number(s.slice(6, 8)));
  return (date.getDay() + 6) % 7;
}

export const WEEKDAY_NAMES = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

export function weekdayLabel(day) {
  return WEEKDAY_NAMES[weekdayMon(day)] || "";
}

export function addDays(day, delta) {
  const s = String(day);
  const date = new Date(Number(s.slice(0, 4)), Number(s.slice(4, 6)) - 1, Number(s.slice(6, 8)));
  date.setDate(date.getDate() + delta);
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return Number(`${y}${m}${d}`);
}

export function diffDays(start, end) {
  const s1 = String(start);
  const s2 = String(end);
  const d1 = new Date(Number(s1.slice(0, 4)), Number(s1.slice(4, 6)) - 1, Number(s1.slice(6, 8)));
  const d2 = new Date(Number(s2.slice(0, 4)), Number(s2.slice(4, 6)) - 1, Number(s2.slice(6, 8)));
  return Math.round((d2 - d1) / 86400000) + 1;
}

export function todayDay() {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  return Number(`${y}${m}${d}`);
}

export function rangeForPreset(preset, customStart, customEnd, today = todayDay()) {
  if (preset === "today") return { start: today, end: today };
  if (preset === "7d") return { start: addDays(today, -6), end: today };
  if (preset === "30d") return { start: addDays(today, -29), end: today };
  if (preset === "90d") return { start: addDays(today, -89), end: today };
  const start = customStart || addDays(today, -6);
  const end = customEnd || today;
  return start <= end ? { start, end } : { start: end, end: start };
}

export function deltaPercent(current, previous) {
  if (previous === null || previous === undefined || previous <= 0) return null;
  return Math.round(((current - previous) * 100) / previous);
}

export function deltaBadge(current, previous) {
  const delta = deltaPercent(current, previous);
  if (delta === null) return { text: "无上期基准", dir: "flat" };
  if (delta > 0) return { text: `+${delta}% 较上期`, dir: "up" };
  if (delta < 0) return { text: `${delta}% 较上期`, dir: "down" };
  return { text: "与上期持平", dir: "flat" };
}

export function fillDays(days, start, end) {
  const map = new Map((days || []).map((d) => [d.day, d]));
  const out = [];
  for (let d = start; d <= end; d = addDays(d, 1)) {
    out.push(
      map.get(d) || {
        day: d,
        physical: 0,
        active_ms: 0,
        backspace: 0,
        delete_key: 0,
        space_key: 0,
        enter_key: 0,
        han: 0,
        latin: 0,
        digit: 0,
        punct: 0,
        commits: 0,
        best_10: 0,
        best_30: 0,
        best_60: 0,
        sessions: 0,
        longest_ms: 0,
        first_minute: -1,
        last_minute: -1
      }
    );
  }
  return out;
}

export const INTERVAL_LABELS = [
  "<30ms",
  "30-50ms",
  "50-80ms",
  "80-120ms",
  "120-200ms",
  "200-300ms",
  "300-500ms",
  "500-800ms",
  "0.8-1.2s",
  "1.2-2.0s",
  "2.0-5.0s",
  ">5.0s"
];

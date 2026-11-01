import { invoke } from "@tauri-apps/api/core";

export async function fetchSnapshot(day = null) {
  return invoke("snapshot", { day });
}

export async function fetchRangeReport(start, end) {
  return invoke("range_report", { start, end });
}

export async function fetchKeyTrend(vk, start, end) {
  return invoke("key_trend", { vk, start, end });
}

export async function fetchTextTrend(table, key, start, end) {
  return invoke("text_trend", { table, key, start, end });
}

export async function setFlag(key, value) {
  return invoke("set_flag", { key, value: String(value) });
}

export async function setIdle(ms) {
  return invoke("set_idle", { ms: Number(ms) });
}

export async function sendCommand(command) {
  return invoke("send_command", { command });
}

export async function setAutostart(on) {
  return invoke("set_autostart", { on: Boolean(on) });
}

export async function openExclude() {
  return invoke("open_exclude");
}

export async function openDataDir() {
  return invoke("open_data_dir");
}

export async function spawnAgent() {
  return invoke("spawn_agent");
}

export async function installRime() {
  return invoke("install_rime");
}

export async function windowControl(action) {
  return invoke("window_control", { action });
}



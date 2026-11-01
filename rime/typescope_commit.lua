-- TypeScope 的提交监听。
-- 任何错误都吞掉，并且永远把按键交回输入法。
-- 只读取已经确认的提交文本，不读取候选词，也不写文件。

local kNoop = 2
local notify = nil
local ready = false

local function load_notify()
  if ready then
    return
  end
  ready = true
  local ok_api, dir = pcall(function()
    return rime_api.get_user_data_dir()
  end)
  if not ok_api or type(dir) ~= "string" then
    return
  end
  local path = dir .. "\\lua\\typescope_rime.dll"
  local ok_load, opener = pcall(package.loadlib, path, "luaopen_typescope")
  if not ok_load or type(opener) ~= "function" then
    return
  end
  local ok_open, mod = pcall(opener)
  if ok_open and type(mod) == "table" and type(mod.notify) == "function" then
    notify = mod.notify
  end
end

local function init(env)
  local ok = pcall(load_notify)
  if not ok or not notify then
    return
  end
  pcall(function()
    env.typescope_conn = env.engine.context.commit_notifier:connect(function(ctx)
      local text = ""
      local got, value = pcall(function()
        return ctx:get_commit_text()
      end)
      if got and type(value) == "string" then
        text = value
      end
      if text ~= "" then
        pcall(notify, text)
      end
    end)
  end)
end

local function fini(env)
  pcall(function()
    if env and env.typescope_conn and env.typescope_conn.disconnect then
      env.typescope_conn:disconnect()
      env.typescope_conn = nil
    end
  end)
end

local function func()
  return kNoop
end

return { init = init, fini = fini, func = func }

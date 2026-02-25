-- Java application request trace with success/error branching
-- Each execution produces a complete request trace (multi-line)

local req_id = uuid()
local user = pick({"alice", "bob", "carol", "dave", "eve"})
local ip = fake_ipv4()
local path = pick({"/api/login", "/api/payments", "/api/users", "/health", "/api/orders"})
local thr = fake_thread_name()
local ts = now_iso()

emit(ts .. " INFO  [" .. thr .. "] [req=" .. req_id .. "] --> " .. fake_http_method() .. " " .. path .. " from " .. user .. "@" .. ip)

-- Simulate processing steps
local steps = int_range(1, 3)
for i = 1, steps do
    local step_ts = now_iso()
    emit(step_ts .. " DEBUG [" .. thr .. "] [req=" .. req_id .. "] processing step " .. i .. "/" .. steps)
end

-- 90% success, 10% error
if weighted_bool(0.9) then
    local elapsed = int_range(5, 250)
    local status = pick({"200", "201", "204"})
    emit(now_iso() .. " INFO  [" .. thr .. "] [req=" .. req_id .. "] <-- " .. status .. " OK elapsed=" .. elapsed .. "ms")
else
    local error_type = pick({"NullPointerException", "RuntimeException", "IOException", "TimeoutException"})
    emit(now_iso() .. " ERROR [" .. thr .. "] [req=" .. req_id .. "] <-- 500 InternalServerError")
    emit(now_iso() .. " ERROR [" .. thr .. "] [req=" .. req_id .. "] " .. error_type .. ": " .. fake_message())

    -- Stack trace
    local java_class = fake_java_class()
    emit("  at " .. java_class .. ".handle(Unknown Source)")
    emit("  at com.app.filter.RequestFilter.doFilter(RequestFilter.java:" .. int_range(20, 150) .. ")")
    emit("  at org.apache.catalina.core.ApplicationFilterChain.doFilter(ApplicationFilterChain.java:166)")

    if weighted_bool(0.5) then
        emit("Caused by: java.io.IOException: " .. pick({"Connection refused", "Connection reset", "Read timed out"}))
        emit("  at java.net.SocketInputStream.read(SocketInputStream.java:" .. int_range(100, 200) .. ")")
    end
end

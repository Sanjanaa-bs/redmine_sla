import json
import requests
from datetime import datetime, timedelta, time, timezone

def parse_aware_datetime(args):
    dt_str = args[0]
    if not dt_str:
        return {"result": None}
    if dt_str.endswith('Z'):
        dt_str = dt_str[:-1] + '+00:00'
    dt = datetime.fromisoformat(dt_str)
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return {"result": dt}

def format_datetime(args):
    dt = args[0]
    if dt is None:
        return {"result": None}
    s = dt.isoformat()
    if s.endswith("+00:00"):
        s = s[:-6] + "Z"
    return {"result": s}

def is_working_day(args):
    dt, working_days_set, holidays_set = args[0], args[1], args[2]
    day_name = dt.strftime("%A").lower()
    if day_name not in working_days_set:
        return {"result": False}
    date_str = dt.strftime("%Y-%m-%d")
    if date_str in holidays_set:
        return {"result": False}
    return {"result": True}

def get_work_start_time(args):
    dt, work_start = args[0], args[1]
    res = dt.replace(hour=work_start.hour, minute=work_start.minute, second=0, microsecond=0)
    return {"result": res}

def get_work_end_time(args):
    dt, work_end = args[0], args[1]
    res = dt.replace(hour=work_end.hour, minute=work_end.minute, second=0, microsecond=0)
    return {"result": res}

def get_next_working_day_start(args):
    dt, work_start = args[0], args[1]
    next_dt = dt + timedelta(days=1)
    res = next_dt.replace(hour=work_start.hour, minute=work_start.minute, second=0, microsecond=0)
    return {"result": res}

def add_working_minutes(args):
    base_dt, minutes, working_days_set, work_start, work_end, holidays_set = args[0], args[1], args[2], args[3], args[4], args[5]
    if minutes is None:
        return {"result": None}
    minutes = int(minutes)
    if minutes <= 0:
        return {"result": base_dt}
    
    current_dt = base_dt
    remaining_minutes = minutes
    limit_days = 365
    day_counter = 0

    while remaining_minutes > 0:
        is_work = is_working_day([current_dt, working_days_set, holidays_set])["result"]
        
        if not is_work:
            current_dt = get_next_working_day_start([current_dt, work_start])["result"]
            day_counter += 1
            if day_counter > limit_days:
                return {"result": None}
            continue

        t = current_dt.time()
        if t < work_start:
            current_dt = get_work_start_time([current_dt, work_start])["result"]
            continue
        elif t >= work_end:
            current_dt = get_next_working_day_start([current_dt, work_start])["result"]
            day_counter += 1
            if day_counter > limit_days:
                return {"result": None}
            continue

        today_end = get_work_end_time([current_dt, work_end])["result"]
        minutes_left_today = int((today_end - current_dt).total_seconds() / 60)
        
        if minutes_left_today <= 0:
            current_dt = get_next_working_day_start([current_dt, work_start])["result"]
            day_counter += 1
            if day_counter > limit_days:
                return {"result": None}
            continue
        
        if remaining_minutes <= minutes_left_today:
            current_dt = current_dt + timedelta(minutes=remaining_minutes)
            remaining_minutes = 0
        else:
            remaining_minutes -= minutes_left_today
            current_dt = today_end

    return {"result": current_dt}

def get_sla_config(args):
    project_id, tracker_id = args[0], args[1]
    url = "http://localhost:8080/api/sla_project_trackers"
    params = {"project_id": project_id, "tracker_id": tracker_id}
    response = requests.get(url, params=params)
    response.raise_for_status()
    data = response.json()
    if isinstance(data, list):
        item = data[0] if len(data) > 0 else {}
    else:
        item = data or {}
    return {
        "sla_id": item.get("sla_id"),
        "schedule_id": item.get("schedule_id")
    }

def get_sla_level(args):
    sla_id, priority = args[0], args[1]
    url = "http://localhost:8080/api/sla_levels"
    params = {"sla_id": sla_id, "priority": priority}
    response = requests.get(url, params=params)
    response.raise_for_status()
    data = response.json()
    if isinstance(data, list):
        item = data[0] if len(data) > 0 else {}
    else:
        item = data or {}
    return {
        "sla_level_id": item.get("sla_level_id"),
        "response_time": item.get("response_time"),
        "resolution_time": item.get("resolution_time")
    }

def get_schedule(args):
    schedule_id = args[0]
    url = f"http://localhost:8080/api/sla_schedules/{schedule_id}"
    response = requests.get(url)
    response.raise_for_status()
    data = response.json()
    return {
        "working_days": data.get("working_days"),
        "start_time": data.get("start_time"),
        "end_time": data.get("end_time"),
        "timezone": data.get("timezone")
    }

def get_holidays(args):
    schedule_id = args[0]
    url = "http://localhost:8080/api/sla_holidays"
    params = {"schedule_id": schedule_id}
    response = requests.get(url, params=params)
    response.raise_for_status()
    data = response.json()
    
    holiday_dates = []
    if isinstance(data, list):
        for item in data:
            if isinstance(item, dict):
                d = item.get("holiday_date") or item.get("date")
                if d:
                    holiday_dates.append(d)
            elif isinstance(item, str):
                holiday_dates.append(item)
    elif isinstance(data, dict):
        holiday_dates = data.get("holiday_dates", [])
        
    cleaned_dates = []
    for d in holiday_dates:
        if isinstance(d, str):
            if 'T' in d:
                cleaned_dates.append(d.split('T')[0])
            else:
                cleaned_dates.append(d)
                
    return {
        "holiday_dates": cleaned_dates
    }

def calculate_deadline(args):
    created_at = args[0]
    response_time = args[1]
    resolution_time = args[2]
    working_days = args[3] or []
    start_time = args[4]
    end_time = args[5]
    holiday_dates = args[6] or []

    start_dt = parse_aware_datetime([created_at])["result"]

    working_days_set = {wd.lower() for wd in working_days}

    holidays_set = set()
    for h in holiday_dates:
        if 'T' in h:
            holidays_set.add(h.split('T')[0])
        else:
            holidays_set.add(h)

    sh, sm = map(int, (start_time or "09:00").split(':'))
    eh, em = map(int, (end_time or "17:00").split(':'))
    work_start = time(sh, sm)
    work_end = time(eh, em)

    resp_res = add_working_minutes([start_dt, response_time, working_days_set, work_start, work_end, holidays_set])
    resp_deadline_dt = resp_res["result"]

    res_res = add_working_minutes([start_dt, resolution_time, working_days_set, work_start, work_end, holidays_set])
    res_deadline_dt = res_res["result"]

    return {
        "response_deadline": format_datetime([resp_deadline_dt])["result"],
        "resolution_deadline": format_datetime([res_deadline_dt])["result"]
    }

def create_sla_cache(args):
    issue_id = args[0]
    project_id = args[1]
    sla_level_id = args[2]
    schedule_id = args[3]
    response_deadline = args[4]
    resolution_deadline = args[5]

    now_str = format_datetime([datetime.now(timezone.utc)])["result"]
    payload = {
        "issue_id": issue_id,
        "project_id": project_id,
        "sla_level_id": sla_level_id,
        "schedule_id": schedule_id,
        "response_deadline": response_deadline,
        "resolution_deadline": resolution_deadline,
        "response_met": False,
        "resolution_met": False,
        "status": "open",
        "created_at": now_str,
        "updated_at": now_str
    }
    response = requests.post("http://localhost:8080/api/sla_caches", json=payload)
    response.raise_for_status()
    data = response.json()
    return {
        "cache_id": data.get("cache_id") or data.get("id") or data.get("_id"),
        "status": data.get("status")
    }

def check_sla_breach(args):
    cache_id = args[0]
    url = f"http://localhost:8080/api/sla_caches/{cache_id}"
    response = requests.get(url)
    response.raise_for_status()
    data = response.json()

    now = datetime.now(timezone.utc)

    response_deadline_str = data.get("response_deadline")
    resolution_deadline_str = data.get("resolution_deadline")

    response_met = data.get("response_met", False)
    resolution_met = data.get("resolution_met", False)

    response_breached = False
    if response_deadline_str and not response_met:
        resp_dl = parse_aware_datetime([response_deadline_str])["result"]
        if now > resp_dl:
            response_breached = True

    resolution_breached = False
    if resolution_deadline_str and not resolution_met:
        res_dl = parse_aware_datetime([resolution_deadline_str])["result"]
        if now > res_dl:
            resolution_breached = True

    if response_breached or resolution_breached:
        status = "breached"
    elif response_met and resolution_met:
        status = "met"
    else:
        status = "open"

    return {
        "cache_id": cache_id,
        "response_breached": response_breached,
        "resolution_breached": resolution_breached,
        "status": status
    }

def update_sla_status(args):
    cache_id = args[0]
    response_breached = args[1]
    resolution_breached = args[2]
    status = args[3]

    payload = {
        "response_met": not response_breached,
        "resolution_met": not resolution_breached,
        "status": status
    }
    url = f"http://localhost:8080/api/sla_caches/{cache_id}"
    response = requests.put(url, json=payload)
    response.raise_for_status()
    data = response.json()
    return {
        "updated": True,
        "final_status": data.get("status") or status
    }

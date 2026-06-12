import json
import requests

PATH = r"c:\Users\sanjana\Downloads\REDMINE\sla_workflow"

def show(label, value):
    print("-" * 50)
    print(label)
    print("-" * 50)
    print(json.dumps(value, indent=2))

def call_step(step_num, function_name, args, workflow_name="sla_workflow"):
    payload = {
        "workflow_name": workflow_name,
        "path": PATH,
        "function": function_name,
        "args": args
    }
    response = requests.post(
        "https://dev.assisto.tech/test/execute_workflow",
        json=payload,
        timeout=60
    )
    if response.status_code != 200:
        print(f"Error calling {function_name}: Status {response.status_code}, Body: {response.text}")
    response.raise_for_status()
    res_data = response.json()
    show(f"Step {step_num}: {function_name}", res_data)
    
    # Handle wrapped result or output if returning from the framework endpoint
    if isinstance(res_data, dict):
        if "result" in res_data and isinstance(res_data["result"], dict):
            return res_data["result"]
        if "output" in res_data and isinstance(res_data["output"], dict):
            return res_data["output"]
    return res_data

def main():
    # Test data
    issue_id = 1001
    project_id = 5
    tracker_id = 1
    priority = "high"
    created_at = "2026-06-12T09:00:00"

    try:
        # WORKFLOW 1 - CONFIGURE SLA
        # Step 1 - create_sla
        res_sla = call_step("1-1", "create_sla", ["Standard SLA", "Default SLA for all projects"], workflow_name="sla_config_workflow")
        sla_id = res_sla.get("sla_id")

        # Step 2 - create_sla_level
        res_level = call_step("1-2", "create_sla_level", [sla_id, "Gold", "high", 240, 1440], workflow_name="sla_config_workflow")
        sla_level_id = res_level.get("sla_level_id")

        # Step 3 - create_sla_schedule
        res_sched = call_step("1-3", "create_sla_schedule", [
            "India Business Hours", "Asia/Kolkata", 
            ["monday","tuesday","wednesday","thursday","friday"],
            "09:00", "18:00"
        ], workflow_name="sla_config_workflow")
        schedule_id = res_sched.get("schedule_id")

        # Step 4 - create_sla_holiday
        res_hol = call_step("1-4", "create_sla_holiday", ["Diwali", "2026-10-20", schedule_id], workflow_name="sla_config_workflow")
        holiday_id = res_hol.get("holiday_id")

        # Step 5 - link_sla_to_project
        call_step("1-5", "link_sla_to_project", [5, 1, sla_id, schedule_id], workflow_name="sla_config_workflow")

        print("Flow 1 Complete - SLA Rules Configured!")

        # Step 1: get_sla_config
        res1 = call_step(1, "get_sla_config", [project_id, tracker_id])
        sla_id = res1.get("sla_id")
        schedule_id = res1.get("schedule_id")

        # Step 2: get_sla_level
        res2 = call_step(2, "get_sla_level", [sla_id, priority])
        sla_level_id = res2.get("sla_level_id")
        response_time = res2.get("response_time")
        resolution_time = res2.get("resolution_time")

        # Step 3: get_schedule
        res3 = call_step(3, "get_schedule", [schedule_id])
        working_days = res3.get("working_days")
        start_time = res3.get("start_time")
        end_time = res3.get("end_time")

        # Step 4: get_holidays
        res4 = call_step(4, "get_holidays", [schedule_id])
        holiday_dates = res4.get("holiday_dates")

        # Step 5: calculate_deadline
        res5 = call_step(5, "calculate_deadline", [
            created_at,
            response_time,
            resolution_time,
            working_days,
            start_time,
            end_time,
            holiday_dates
        ])
        response_deadline = res5.get("response_deadline")
        resolution_deadline = res5.get("resolution_deadline")

        # Step 6: create_sla_cache
        res6 = call_step(6, "create_sla_cache", [
            issue_id,
            project_id,
            sla_level_id,
            schedule_id,
            response_deadline,
            resolution_deadline
        ])
        cache_id = res6.get("cache_id")

        # Step 7: check_sla_breach
        res7 = call_step(7, "check_sla_breach", [cache_id])
        response_breached = res7.get("response_breached")
        resolution_breached = res7.get("resolution_breached")
        status = res7.get("status")

        # Step 8: update_sla_status
        call_step(8, "update_sla_status", [
            cache_id,
            response_breached,
            resolution_breached,
            status
        ])

        print("SLA Workflow Complete!")

    except Exception as e:
        print(f"Error during workflow execution: {e}")

if __name__ == "__main__":
    main()

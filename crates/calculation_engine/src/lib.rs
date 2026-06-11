use chrono::{DateTime, Utc};
use models::{SlaDefinition, Calendar, SpentStatusTime, StatusInterval as ModelStatusInterval};
use transition_reconstructor::StatusInterval;

pub struct CalculationResult {
    pub level_name: String,
    pub response_deadline: Option<DateTime<Utc>>,
    pub resolution_deadline: Option<DateTime<Utc>>,
    pub response_met: bool,
    pub resolution_met: bool,
    pub status: String,
    pub spent_minutes_by_status: Vec<SpentStatusTime>,
}

pub fn calculate_sla(
    sla_def: &SlaDefinition,
    calendar: &Calendar,
    intervals: &[StatusInterval],
    priority: &str,
    current_time: DateTime<Utc>,
) -> Option<CalculationResult> {
    if intervals.is_empty() {
        return None;
    }

    // 1. Find matching SLA Level by priority
    let level = sla_def.levels.iter().find(|l| {
        l.priority.to_lowercase() == priority.to_lowercase()
    })?;

    let created_at = intervals.first()?.started_at;

    // 2. Calculate Deadlines using calendar_engine
    let response_deadline = if level.response_time > 0 {
        Some(calendar_engine::add_working_minutes(created_at, level.response_time as i64, calendar))
    } else {
        None
    };

    let resolution_deadline = if level.resolution_time > 0 {
        Some(calendar_engine::add_working_minutes(created_at, level.resolution_time as i64, calendar))
    } else {
        None
    };

    // 3. Calculate spent minutes and group by status
    let mut spent_by_status: std::collections::HashMap<i32, (String, i32, Vec<ModelStatusInterval>)> = std::collections::HashMap::new();
    let mut first_status_ended_at: Option<DateTime<Utc>> = None;

    for (index, interval) in intervals.iter().enumerate() {
        let is_active = sla_def.active_statuses.contains(&interval.status_id);
        let end_time = interval.ended_at.unwrap_or(current_time);
        
        if index == 0 {
            first_status_ended_at = interval.ended_at;
        }

        let time_spent = if is_active {
            calendar_engine::calculate_working_minutes(interval.started_at, end_time, calendar) as i32
        } else {
            0
        };

        let entry = spent_by_status.entry(interval.status_id).or_insert_with(|| {
            (interval.status_name.clone(), 0, Vec::new())
        });
        
        entry.1 += time_spent;
        entry.2.push(ModelStatusInterval {
            started_at: interval.started_at,
            ended_at: interval.ended_at,
        });
    }

    // Convert map to Vec<SpentStatusTime>
    let spent_minutes_by_status: Vec<SpentStatusTime> = spent_by_status
        .into_iter()
        .map(|(status_id, (status_name, time_spent, intervals))| SpentStatusTime {
            status_id,
            status_name,
            time_spent,
            intervals,
        })
        .collect();

    // 4. Determine if response target is met
    let response_met = match (response_deadline, first_status_ended_at) {
        (Some(deadline), Some(ended)) => ended <= deadline,
        (Some(deadline), None) => current_time <= deadline,
        _ => true,
    };

    // 5. Determine if resolution target is met
    // Find when the ticket entered a non-active status
    let mut resolved_at: Option<DateTime<Utc>> = None;
    for interval in intervals {
        let is_active = sla_def.active_statuses.contains(&interval.status_id);
        if !is_active {
            resolved_at = Some(interval.started_at);
            break;
        }
    }

    let resolution_met = match (resolution_deadline, resolved_at) {
        (Some(deadline), Some(resolved)) => resolved <= deadline,
        (Some(deadline), None) => current_time <= deadline,
        _ => true,
    };

    // Determine overall cache status
    let status = if !response_met || !resolution_met {
        "Breached".to_string()
    } else if resolved_at.is_some() {
        "Completed".to_string()
    } else {
        "Active".to_string()
    };

    Some(CalculationResult {
        level_name: level.name.clone(),
        response_deadline,
        resolution_deadline,
        response_met,
        resolution_met,
        status,
        spent_minutes_by_status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::SlaLevel;

    #[test]
    fn test_calculate_sla() {
        let calendar = Calendar {
            id: models::ObjectId::new(),
            name: "Standard 9-5".to_string(),
            timezone: "UTC".to_string(),
            working_days: vec!["Monday".to_string(), "Tuesday".to_string(), "Wednesday".to_string(), "Thursday".to_string(), "Friday".to_string()],
            start_time: "09:00".to_string(),
            end_time: "17:00".to_string(),
            holidays: vec![],
        };

        let sla_def = SlaDefinition {
            id: models::ObjectId::new(),
            name: "Silver SLA".to_string(),
            description: None,
            sla_type: "Response & Resolution".to_string(),
            levels: vec![SlaLevel {
                name: "High Priority Level".to_string(),
                response_time: 60,      // 1 hour
                resolution_time: 480,   // 8 hours (1 full working day)
                priority: "High".to_string(),
            }],
            active_statuses: vec![1, 2], // 1 = New, 2 = In Progress. Resolved status (3) is not active.
            project_tracker_mappings: vec![],
            calendar_id: calendar.id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created_at = DateTime::parse_from_rfc3339("2026-01-02T09:00:00Z").unwrap().with_timezone(&Utc); // Friday 9 AM
        let resolved_at = DateTime::parse_from_rfc3339("2026-01-02T15:00:00Z").unwrap().with_timezone(&Utc); // Friday 3 PM (6 working hours later)
        
        let intervals = vec![
            StatusInterval {
                status_id: 1,
                status_name: "New".to_string(),
                started_at: created_at,
                ended_at: Some(created_at + chrono::Duration::minutes(30)), // Responded in 30 mins (less than 60 mins target)
            },
            StatusInterval {
                status_id: 2,
                status_name: "In Progress".to_string(),
                started_at: created_at + chrono::Duration::minutes(30),
                ended_at: Some(resolved_at),
            },
            StatusInterval {
                status_id: 3,
                status_name: "Resolved".to_string(),
                started_at: resolved_at,
                ended_at: None,
            }
        ];

        let result = calculate_sla(
            &sla_def,
            &calendar,
            &intervals,
            "High",
            resolved_at,
        ).unwrap();

        assert_eq!(result.level_name, "High Priority Level");
        assert!(result.response_met);
        assert!(result.resolution_met);
        assert_eq!(result.status, "Completed");

        // Total active spent time should be 360 minutes (6 hours)
        let new_spent = result.spent_minutes_by_status.iter().find(|s| s.status_id == 1).unwrap().time_spent;
        let ip_spent = result.spent_minutes_by_status.iter().find(|s| s.status_id == 2).unwrap().time_spent;
        assert_eq!(new_spent + ip_spent, 360);
    }
}

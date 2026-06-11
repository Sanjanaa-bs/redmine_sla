use chrono::{DateTime, Utc};
use models::Calendar;

pub fn calculate_working_minutes(
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
    calendar: &Calendar,
) -> i64 {
    if started_at >= ended_at {
        return 0;
    }

    let (start_h, start_m) = parse_time(&calendar.start_time);
    let (end_h, end_m) = parse_time(&calendar.end_time);

    let mut total_minutes = 0;
    
    let start_day = started_at.date_naive();
    let end_day = ended_at.date_naive();

    let mut current_day = start_day;
    while current_day <= end_day {
        let weekday_str = current_day.format("%A").to_string();
        let is_working_day = calendar.working_days.contains(&weekday_str);
        
        let is_holiday = calendar.holidays.iter().any(|h| {
            h.date.date_naive() == current_day
        });

        if is_working_day && !is_holiday {
            // Construct working hour bounds for current day in UTC
            if let (Some(work_start), Some(work_end)) = (
                current_day.and_hms_opt(start_h, start_m, 0),
                current_day.and_hms_opt(end_h, end_m, 0)
            ) {
                let intersect_start = std::cmp::max(started_at.naive_utc(), work_start);
                let intersect_end = std::cmp::min(ended_at.naive_utc(), work_end);

                if intersect_end > intersect_start {
                    total_minutes += (intersect_end - intersect_start).num_minutes();
                }
            }
        }
        
        // Advance to next day
        if let Some(next_day) = current_day.checked_add_days(chrono::Days::new(1)) {
            current_day = next_day;
        } else {
            break;
        }
    }

    total_minutes
}

fn parse_time(time_str: &str) -> (u32, u32) {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 2 {
        let h = parts[0].parse().unwrap_or(9);
        let m = parts[1].parse().unwrap_or(0);
        (h, m)
    } else {
        (9, 0)
    }
}

// Function to add working minutes to a date
pub fn add_working_minutes(
    start_dt: DateTime<Utc>,
    minutes_to_add: i64,
    calendar: &Calendar,
) -> DateTime<Utc> {
    if minutes_to_add <= 0 {
        return start_dt;
    }

    let (start_h, start_m) = parse_time(&calendar.start_time);
    let (end_h, end_m) = parse_time(&calendar.end_time);

    let mut current_dt = start_dt;
    let mut minutes_remaining = minutes_to_add;

    // We loop and advance the time day by day or hour by hour
    while minutes_remaining > 0 {
        let current_day = current_dt.date_naive();
        let weekday_str = current_day.format("%A").to_string();
        let is_working_day = calendar.working_days.contains(&weekday_str);
        
        let is_holiday = calendar.holidays.iter().any(|h| {
            h.date.date_naive() == current_day
        });

        if is_working_day && !is_holiday {
            let work_start = current_day.and_hms_opt(start_h, start_m, 0).unwrap();
            let work_end = current_day.and_hms_opt(end_h, end_m, 0).unwrap();

            let active_start = std::cmp::max(current_dt.naive_utc(), work_start);
            
            if active_start < work_end {
                let available_minutes = (work_end - active_start).num_minutes();
                if minutes_remaining <= available_minutes {
                    // Fits in today's working window
                    let final_naive = active_start + chrono::Duration::minutes(minutes_remaining);
                    return DateTime::<Utc>::from_naive_utc_and_offset(final_naive, Utc);
                } else {
                    // Consume today's working window
                    minutes_remaining -= available_minutes;
                    current_dt = DateTime::<Utc>::from_naive_utc_and_offset(work_end, Utc);
                }
            }
        }

        // Advance to start of next working day
        if let Some(next_day) = current_day.checked_add_days(chrono::Days::new(1)) {
            let next_work_start = next_day.and_hms_opt(start_h, start_m, 0).unwrap();
            current_dt = DateTime::<Utc>::from_naive_utc_and_offset(next_work_start, Utc);
        } else {
            break;
        }
    }

    current_dt
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::Holiday;

    #[test]
    fn test_calculate_working_minutes() {
        let calendar = Calendar {
            id: models::ObjectId::new(),
            name: "Standard 9-5".to_string(),
            timezone: "UTC".to_string(),
            working_days: vec![
                "Monday".to_string(),
                "Tuesday".to_string(),
                "Wednesday".to_string(),
                "Thursday".to_string(),
                "Friday".to_string(),
            ],
            start_time: "09:00".to_string(),
            end_time: "17:00".to_string(),
            holidays: vec![
                Holiday {
                    name: "New Year".to_string(),
                    date: DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
                }
            ],
        };

        // Same day within working hours
        let start = DateTime::parse_from_rfc3339("2026-01-02T10:00:00Z").unwrap().with_timezone(&Utc); // Friday
        let end = DateTime::parse_from_rfc3339("2026-01-02T12:00:00Z").unwrap().with_timezone(&Utc);
        assert_eq!(calculate_working_minutes(start, end, &calendar), 120);

        // Same day partially outside working hours
        let start = DateTime::parse_from_rfc3339("2026-01-02T08:00:00Z").unwrap().with_timezone(&Utc); // before 9am
        let end = DateTime::parse_from_rfc3339("2026-01-02T18:00:00Z").unwrap().with_timezone(&Utc);  // after 5pm
        assert_eq!(calculate_working_minutes(start, end, &calendar), 480); // 8 hours = 480 mins

        // Over a weekend (Friday 16:00 to Monday 10:00)
        let start = DateTime::parse_from_rfc3339("2026-01-02T16:00:00Z").unwrap().with_timezone(&Utc); // Fri 4pm
        let end = DateTime::parse_from_rfc3339("2026-01-05T10:00:00Z").unwrap().with_timezone(&Utc);   // Mon 10am
        // Should be 1 hour of Friday (16:00 - 17:00) + 1 hour of Monday (09:00 - 10:00) = 120 mins
        assert_eq!(calculate_working_minutes(start, end, &calendar), 120);

        // Over a holiday (Wednesday Dec 31 to Friday Jan 02) with Jan 01 as holiday
        let start = DateTime::parse_from_rfc3339("2025-12-31T16:00:00Z").unwrap().with_timezone(&Utc); // Wed 4pm (1 hr left)
        let end = DateTime::parse_from_rfc3339("2026-01-02T10:00:00Z").unwrap().with_timezone(&Utc);   // Fri 10am (1 hr spent)
        // Thursday Jan 01 is Holiday. Total = 60 + 60 = 120 mins
        assert_eq!(calculate_working_minutes(start, end, &calendar), 120);
    }

    #[test]
    fn test_add_working_minutes() {
        let calendar = Calendar {
            id: models::ObjectId::new(),
            name: "Standard 9-5".to_string(),
            timezone: "UTC".to_string(),
            working_days: vec![
                "Monday".to_string(),
                "Tuesday".to_string(),
                "Wednesday".to_string(),
                "Thursday".to_string(),
                "Friday".to_string(),
            ],
            start_time: "09:00".to_string(),
            end_time: "17:00".to_string(),
            holidays: vec![],
        };

        // Add 60 minutes during active day
        let start = DateTime::parse_from_rfc3339("2026-01-02T10:00:00Z").unwrap().with_timezone(&Utc);
        let result = add_working_minutes(start, 60, &calendar);
        assert_eq!(result.to_rfc3339(), "2026-01-02T11:00:00+00:00");

        // Add 60 minutes at end of day, pushing to next working day morning
        let start = DateTime::parse_from_rfc3339("2026-01-02T16:30:00Z").unwrap().with_timezone(&Utc); // Fri 4:30pm
        let result = add_working_minutes(start, 60, &calendar);
        // Friday gets 30 minutes (until 17:00). Next 30 minutes are applied on Monday 09:00 -> 09:30
        assert_eq!(result.to_rfc3339(), "2026-01-05T09:30:00+00:00");
    }
}

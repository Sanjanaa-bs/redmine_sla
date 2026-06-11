use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transition {
    pub status_id: i32,
    pub status_name: String,
    pub transitioned_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusInterval {
    pub status_id: i32,
    pub status_name: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

pub fn reconstruct_intervals(
    created_at: DateTime<Utc>,
    initial_status_id: i32,
    initial_status_name: String,
    mut transitions: Vec<Transition>,
    _current_time: DateTime<Utc>,
) -> Vec<StatusInterval> {
    // Sort transitions chronologically
    transitions.sort_by_key(|t| t.transitioned_at);

    let mut intervals = Vec::new();
    
    let mut current_status_id = initial_status_id;
    let mut current_status_name = initial_status_name;
    let mut last_time = created_at;

    for t in transitions {
        if t.transitioned_at <= last_time {
            continue;
        }

        // Close the previous interval
        intervals.push(StatusInterval {
            status_id: current_status_id,
            status_name: current_status_name.clone(),
            started_at: last_time,
            ended_at: Some(t.transitioned_at),
        });

        // Start new interval
        current_status_id = t.status_id;
        current_status_name = t.status_name;
        last_time = t.transitioned_at;
    }

    // Add the final ongoing active interval
    intervals.push(StatusInterval {
        status_id: current_status_id,
        status_name: current_status_name,
        started_at: last_time,
        ended_at: None, // Still active
    });

    intervals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconstruct_intervals() {
        let created_at = DateTime::parse_from_rfc3339("2026-01-01T10:00:00Z").unwrap().with_timezone(&Utc);
        let t1_time = DateTime::parse_from_rfc3339("2026-01-01T12:00:00Z").unwrap().with_timezone(&Utc);
        let t2_time = DateTime::parse_from_rfc3339("2026-01-01T15:00:00Z").unwrap().with_timezone(&Utc);
        let current_time = DateTime::parse_from_rfc3339("2026-01-01T18:00:00Z").unwrap().with_timezone(&Utc);

        let transitions = vec![
            Transition {
                status_id: 2,
                status_name: "In Progress".to_string(),
                transitioned_at: t1_time,
            },
            Transition {
                status_id: 3,
                status_name: "Resolved".to_string(),
                transitioned_at: t2_time,
            },
        ];

        let intervals = reconstruct_intervals(
            created_at,
            1,
            "New".to_string(),
            transitions,
            current_time,
        );

        assert_eq!(intervals.len(), 3);

        // Interval 1: New
        assert_eq!(intervals[0].status_name, "New");
        assert_eq!(intervals[0].started_at, created_at);
        assert_eq!(intervals[0].ended_at, Some(t1_time));

        // Interval 2: In Progress
        assert_eq!(intervals[1].status_name, "In Progress");
        assert_eq!(intervals[1].started_at, t1_time);
        assert_eq!(intervals[1].ended_at, Some(t2_time));

        // Interval 3: Resolved (ongoing)
        assert_eq!(intervals[2].status_name, "Resolved");
        assert_eq!(intervals[2].started_at, t2_time);
        assert_eq!(intervals[2].ended_at, None);
    }
}

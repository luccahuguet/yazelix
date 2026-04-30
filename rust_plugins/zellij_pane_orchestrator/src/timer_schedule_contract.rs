use std::time::{Duration, Instant};

const MINIMUM_TIMER_DELAY: Duration = Duration::from_millis(500);

pub fn next_timer_delay<I>(
    now: Instant,
    deadlines: I,
    armed_deadline: Option<Instant>,
) -> Option<(Instant, Duration)>
where
    I: IntoIterator<Item = Option<Instant>>,
{
    let next_deadline = deadlines.into_iter().flatten().min()?;
    if let Some(armed_deadline) = armed_deadline {
        if armed_deadline <= next_deadline {
            return None;
        }
    }

    let delay = next_deadline.saturating_duration_since(now);
    Some((next_deadline, delay.max(MINIMUM_TIMER_DELAY)))
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::{next_timer_delay, MINIMUM_TIMER_DELAY};
    use std::time::{Duration, Instant};

    // Regression: multiple orchestrator refresh loops must share one Zellij timeout instead of arming a timeout per non-due loop.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn arms_only_the_earliest_unarmed_deadline() {
        let now = Instant::now();
        let early = now + Duration::from_secs(2);
        let later = now + Duration::from_secs(120);

        assert_eq!(
            next_timer_delay(now, [Some(later), None, Some(early)], None),
            Some((early, Duration::from_secs(2)))
        );
        assert_eq!(
            next_timer_delay(now, [Some(later), None, Some(early)], Some(early)),
            None
        );
        assert_eq!(next_timer_delay(now, [Some(later)], Some(early)), None);
    }

    // Defends: overdue work is handled promptly without asking Zellij for sub-frame timer churn.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn clamps_overdue_deadlines_to_minimum_delay() {
        let now = Instant::now();
        let overdue = now - Duration::from_secs(3);

        assert_eq!(
            next_timer_delay(now, [Some(overdue)], None),
            Some((overdue, MINIMUM_TIMER_DELAY))
        );
    }
}

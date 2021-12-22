use std::{fmt, time::Duration};

pub struct TimingStatistic {
    pub duration: Duration,
    pub description: String,
}

impl fmt::Display for TimingStatistic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.3?}s {}",
            self.duration.as_secs_f32(),
            self.description
        )
    }
}

#[macro_export]
macro_rules! display_timings {
    ($( $t: expr),*) => {
        vec![
            $(
                $crate::display_timings::TimingStatistic {
                duration: $t.0,
                description: $t.1.to_string()
                },
            )*
        ]
        .iter()
        .map(|ts| format!("{}", ts.to_string()))
        .collect::<Vec<String>>()
        .join("\n")
    }
}

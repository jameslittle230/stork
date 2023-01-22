use std::{fmt, time::Duration};

pub struct TimingStatistic {
    pub duration: Duration,
    pub description: String,
}

impl fmt::Display for TimingStatistic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #![allow(clippy::cast_precision_loss)]
        write!(
            f,
            "{:.3?} ms {}",
            self.duration.as_micros() as f64 / 1000.0,
            self.description
        )
    }
}

#[macro_export]
macro_rules! print {
    ($( $t: expr),*) => {
        eprintln!(
            "{}",
            vec![
                $(
                    $crate::timings::TimingStatistic {
                    duration: $t.0,
                    description: $t.1.to_string()
                    },
                )*
            ]
            .iter()
            .map(|ts| format!("{}", ts.to_string()))
            .collect::<Vec<String>>()
            .join("\n")
        )
    }
}

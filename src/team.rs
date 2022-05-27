use std::{
    cmp::Ordering,
    time::{Duration, Instant},
};

use tui::widgets::Row;

pub struct Team {
    pub id: u32,
    name: String,
    start_time: String,
    race_time: String,
    running_time: Option<Instant>,
}

impl Team {
    pub fn new(id: u32, name: String, start_time: String) -> Self {
        Self {
            id,
            name,
            start_time,
            race_time: String::new(),
            running_time: None,
        }
    }

    pub fn start_stop_timer(&mut self) {
        match self.running_time {
            Some(t) => {
                self.race_time = format_time(t.elapsed());
                self.running_time = None;
            }
            None => {
                if self.race_time.len() > 0 {
                    return;
                }

                self.running_time = Some(Instant::now())
            }
        };
    }

    pub fn reset_time(&mut self) {
        self.running_time = None;
        self.race_time = String::new();
    }

    pub fn get_time(&self) -> String {
        if let Some(i) = self.running_time {
            format_time(i.elapsed())
        } else {
            self.race_time.clone()
        }
    }
}

impl Into<Row<'_>> for &Team {
    fn into(self) -> Row<'static> {
        Row::new::<Vec<String>>(vec![
            format!("#{}", self.id),
            self.name.clone(),
            self.start_time.clone(),
            self.get_time(),
        ])
    }
}

impl Ord for Team {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if &self.race_time == "" {
            return Ordering::Greater;
        }

        if &other.race_time == "" {
            return Ordering::Less;
        }

        self.race_time.cmp(&other.race_time)
    }
}

impl PartialOrd for Team {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Team {}

fn format_time(d: Duration) -> String {
    format!(
        "{}:{}:{}",
        format_count(d.as_secs() / 60),
        format_count(d.as_secs() % 60),
        format_count(d.subsec_millis() as u64)
    )
}

fn format_count(time: u64) -> String {
    let mut unf = time.clone().to_string();
    unf.truncate(2);

    if unf.len() == 1 {
        return format!("0{}", unf);
    } else {
        return format!("{}", unf);
    }
}

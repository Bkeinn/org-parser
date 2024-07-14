use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use regex::Regex;

#[derive(Debug, Clone, Copy)]
enum Repeat {
    Dayly,
    Weekly,
    Monthly,
    Yearly,
}

impl Repeat {
    fn build(&self) -> char {
        match self {
            Repeat::Dayly => 'd',
            Repeat::Weekly => 'w',
            Repeat::Monthly => 'm',
            Repeat::Yearly => 'y',
        }
    }
}

/// Saves dates in an org document
#[derive(Debug, Clone)]
pub struct ParsedDateTime {
    date: NaiveDate,
    day: String,
    repeat: Option<Repeat>,
}
/// Returns true when the date was changed, false if it was not changed
impl ParsedDateTime {
    /// Updates a ParsedDateTime, by it's repeater if the date passed todays date
    pub fn update(&mut self) -> bool {
        let tody = chrono::Utc::now().date_naive();
        if self.date < tody {
            if let Some(repeater) = self.repeat {
                match repeater {
                    Repeat::Dayly => self.date = self.date + Duration::days(1),
                    Repeat::Weekly => self.date = self.date + Duration::weeks(1),
                    Repeat::Monthly => self.date = self.date + Duration::days(30),// Months are now 30 days, fact, pull request if you have a better idea
                    Repeat::Yearly => self.date = self.date + Duration::days(365),// Fuck Schaltjahre
                }
                return true;
            }
        }
        return false;
    }
    /// Creates a String representation of the Time, allways in <>
    pub fn build(&self) -> String {
        let repeat = match &self.repeat {
            Some(rep) => format!(" .+l{}", rep.build()),
            None => "".to_owned(),
        };
        return format!("<{} {}{repeat}>", self.date.to_string(), self.day);
    }
    /// Parses a org time into this struct, returns None if it could not parse
    pub fn parse(input: &str) -> Option<Self> {
        let re = Regex::new(r"<(\d{4}-\d{2}-\d{2}) (\w{3})( \.\+l(d|w|m|y)|)>").unwrap();

        if let Some(caps) = re.captures(input) {
            let date_str = caps.get(1)?.as_str();
            let day_str = caps.get(2)?.as_str().to_string();
            let repeat = match caps.get(3)?.as_str().chars().last() {
                Some(character) => match character {
                    'd' => Some(Repeat::Dayly),
                    'w' => Some(Repeat::Weekly),
                    'm' => Some(Repeat::Monthly),
                    'y' => Some(Repeat::Yearly),
                    _ => None,
                },
                None => None,
            };

            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;

            Some(ParsedDateTime {
                date,
                repeat,
                day: day_str,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
/// Information types, often given underneath a heading
pub enum InfoType {
    SCHEDULED { date: ParsedDateTime },
    DEADLINE { date: ParsedDateTime },
}

impl InfoType {
    /// Does only want one potential InfoType at a time as this funktion does not export Vec
    pub fn get(input: &str) -> Option<InfoType> {
        let re_dead = Regex::new("DEADLINE:").unwrap();
        let re_sche = Regex::new("SCHEDULED:").unwrap();
        if re_dead.is_match(input) {
            return Some(InfoType::DEADLINE {
                date: match ParsedDateTime::parse(input) {
                    Some(datetime) => datetime,
                    None => return None,
                },
            });
        } else if re_sche.is_match(input) {
            return Some(InfoType::SCHEDULED {
                date: match ParsedDateTime::parse(input) {
                    Some(datetime) => datetime,
                    None => return None,
                },
            });
        } else {
            return None;
        }
    }
    pub fn build(self) -> String {
        match self {
            InfoType::DEADLINE { date } => format!("DEADLINE: {}", date.build()),
            InfoType::SCHEDULED { date } => format!("SCHEDULED: {}", date.build()),
        }
    }
}

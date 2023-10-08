use anyhow::anyhow;
use chrono::{offset::TimeZone, DateTime, Datelike, NaiveTime};
use chrono_tz::Europe::Kiev;

#[derive(Debug)]
pub struct Subject {
    pub id: i64,
    pub title: String,
    pub group: Group,
    pub optional: bool,
}

#[derive(Debug)]
pub struct Meeting {
    pub id: i64,
    pub name: String,
    pub group: Group,
    pub link: String,
}

#[derive(Debug)]
pub struct Schedule {
    pub subject_id: i64,
    pub day: Day,
    pub repeat: Repeat,
    pub slot: Slot,
}

#[derive(Debug)]
pub struct Assigned {
    pub meeting_id: i64,
    pub subject_id: i64,
}

#[derive(PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Repeat {
    Odd = 0b01,
    Even = 0b10,
    Both = 0b11,
}

impl TryFrom<&str> for Repeat {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use Repeat::*;

        match value {
            "Odd" => Ok(Odd),
            "Even" => Ok(Even),
            "Both" => Ok(Both),
            other => Err(anyhow!("Not a repeat: {}", other)),
        }
    }
}

impl<Tz: TimeZone> From<&DateTime<Tz>> for Repeat {
    fn from(value: &DateTime<Tz>) -> Self {
        let time = value.with_timezone(&Kiev);
        let is_odd = ((Datelike::day(&time) + 6) / 7 % 2) == 0;
        if is_odd {
            Repeat::Odd
        } else {
            Repeat::Even
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Group {
    K25,
}

impl TryFrom<&str> for Group {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use Group::*;
        match value {
            "K-25" => Ok(K25),
            other => Err(anyhow!("Not a group: {}", other)),
        }
    }
}

impl From<&Group> for String {
    fn from(value: &Group) -> Self {
        use Group::*;
        match value {
            K25 => "K-25",
        }
        .into()
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Day {
    Mon = 1,
    Tue = 2,
    Wed = 3,
    Thu = 4,
    Fri = 5,
}

impl TryFrom<&str> for Day {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use Day::*;

        match value {
            "Mon" => Ok(Mon),
            "Tue" => Ok(Tue),
            "Wed" => Ok(Wed),
            "Thu" => Ok(Thu),
            "Fri" => Ok(Fri),
            other => Err(anyhow!("Not a day: {}", other)),
        }
    }
}

impl<Tz: TimeZone> TryFrom<&DateTime<Tz>> for Day {
    type Error = anyhow::Error;
    fn try_from(value: &DateTime<Tz>) -> Result<Self, Self::Error> {
        let time = value.with_timezone(&Kiev);
        use chrono::Weekday::*;
        match time.weekday() {
            Mon => Ok(Day::Mon),
            Tue => Ok(Day::Tue),
            Wed => Ok(Day::Wed),
            Thu => Ok(Day::Thu),
            Fri => Ok(Day::Fri),
            other => Err(anyhow::anyhow!("{} is not a valid weekday", other)),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Slot {
    I = 1,
    II = 2,
    III = 3,
    IV = 4,
}

impl TryFrom<&str> for Slot {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use Slot::*;

        match value {
            "1" | "I" => Ok(I),
            "2" | "II" => Ok(II),
            "3" | "III" => Ok(III),
            "4" | "IV" => Ok(IV),
            other => Err(anyhow!("Not a slot: {}", other)),
        }
    }
}

impl<Tz: TimeZone> From<&DateTime<Tz>> for Slot {
    fn from(value: &DateTime<Tz>) -> Self {
        let time = value.with_timezone(&Kiev).time();
        let ends = [
            (NaiveTime::from_hms_opt(10, 15, 0).unwrap(), Slot::I),
            (NaiveTime::from_hms_opt(12, 10, 0).unwrap(), Slot::II),
            (NaiveTime::from_hms_opt(13, 55, 0).unwrap(), Slot::III),
            (NaiveTime::from_hms_opt(15, 40, 0).unwrap(), Slot::IV),
        ];
        for (end, slot) in ends {
            if time < end {
                return slot;
            }
        }

        return Slot::I;
    }
}

pub trait Unpackable {
    fn unpack<I: IntoIterator<Item = String>>(packed: I) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl Unpackable for Subject {
    fn unpack<I: IntoIterator<Item = String>>(input: I) -> anyhow::Result<Subject> {
        let mut iter = input.into_iter();
        let mut next = |field| match iter.next() {
            Some(some) => Ok(some),
            None => Err(anyhow!("Missing value for {}", field)),
        };

        let id = {
            let value = next("id")?;
            i64::from_str_radix(value.as_ref(), 10)?
        };
        let title = next("title")?;
        let group = Group::try_from(next("group")?.as_str())?;
        let optional = match next("optional")?.as_ref() {
            "true" => true,
            _ => false,
        };

        if let Some(extra) = iter.next() {
            if !extra.is_empty() {
                log::warn!("Found extra value while unpacking `Subject`: '{}'", extra);
            }
        }

        Ok(Subject {
            id,
            title,
            group,
            optional,
        })
    }
}

impl Unpackable for Meeting {
    fn unpack<I: IntoIterator<Item = String>>(input: I) -> anyhow::Result<Meeting> {
        let mut iter = input.into_iter();
        let mut next = |field| match iter.next() {
            Some(some) => Ok(some),
            None => Err(anyhow!("Missing value for {}", field)),
        };

        let id = {
            let value = next("id")?;
            i64::from_str_radix(value.as_ref(), 10)?
        };
        let name = next("name")?;
        let group = Group::try_from(next("group")?.as_str())?;
        let link = next("link")?;

        if let Some(extra) = iter.next() {
            if !extra.is_empty() {
                log::warn!("Found extra value while unpacking `Meeting`: '{}'", extra);
            }
        }

        Ok(Meeting {
            id,
            name,
            group,
            link,
        })
    }
}

impl Unpackable for Schedule {
    fn unpack<I: IntoIterator<Item = String>>(input: I) -> anyhow::Result<Schedule> {
        let mut iter = input.into_iter();
        let mut next = |field| match iter.next() {
            Some(some) => Ok(some),
            None => Err(anyhow!("Missing value for {}", field)),
        };

        let day = Day::try_from(next("day")?.as_str())?;
        let subject_id = {
            let value = next("subject_id")?;
            i64::from_str_radix(value.as_ref(), 10)?
        };
        let repeat = Repeat::try_from(next("repeat")?.as_str())?;
        let slot = Slot::try_from(next("slot")?.as_str())?;

        if let Some(extra) = iter.next() {
            if !extra.is_empty() {
                log::warn!("Found extra value while unpacking `Schedule`: '{}'", extra);
            }
        }

        Ok(Schedule {
            subject_id,
            day,
            repeat,
            slot,
        })
    }
}

impl Unpackable for Assigned {
    fn unpack<I: IntoIterator<Item = String>>(input: I) -> anyhow::Result<Assigned> {
        let mut iter = input.into_iter();
        let mut next = |field| match iter.next() {
            Some(some) => Ok(some),
            None => Err(anyhow!("Missing value for {}", field)),
        };

        let meeting_id = {
            let value = next("meeting_id")?;
            i64::from_str_radix(value.as_ref(), 10)?
        };
        let subject_id = {
            let value = next("subject_id")?;
            i64::from_str_radix(value.as_ref(), 10)?
        };

        if let Some(extra) = iter.next() {
            if !extra.is_empty() {
                log::warn!("Found extra value while unpacking `Assigned`: '{}'", extra);
            }
        }

        Ok(Assigned {
            meeting_id,
            subject_id,
        })
    }
}

pub fn unpack<P: AsRef<std::path::Path>, U: Unpackable>(
    path: P,
    fields: usize,
) -> anyhow::Result<Vec<U>> {
    let lines: Vec<String> = std::fs::read_to_string(path)?
        .lines()
        .map(String::from)
        .collect();
    let mut unpacked = vec![];
    for chunk in lines.chunks(fields + 1) {
        match U::unpack(chunk.into_iter().cloned()) {
            Ok(value) => {
                unpacked.push(value);
            }
            Err(error) => {
                log::error!("{}", error.to_string());
            }
        }
    }

    Ok(unpacked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_unpacks() {
        let packed = vec![
            "0".into(),
            "Test title".into(),
            "K-25".into(),
            "false".into(),
        ];

        let unpacked = Subject::unpack(packed).expect("Failed to unpack test data");

        assert_eq!(unpacked.id, 0);
        assert_eq!(unpacked.title, "Test title");
        assert_eq!(unpacked.group, Group::K25);
        assert_eq!(unpacked.optional, false);
    }

    #[test]
    fn meeting_unpacks() {
        let packed = vec![
            "0".into(),
            "Test name".into(),
            "K-25".into(),
            "https://fake-link.lol".into(),
        ];

        let unpacked = Meeting::unpack(packed).expect("Failed to unpack test data");

        assert_eq!(unpacked.id, 0);
        assert_eq!(unpacked.name, "Test name");
        assert_eq!(unpacked.group, Group::K25);
        assert_eq!(unpacked.link, "https://fake-link.lol");
    }

    #[test]
    fn schedule_unpacks() {
        let packed = vec!["Mon".into(), "0".into(), "Both".into(), "4".into()];

        let unpacked = Schedule::unpack(packed).expect("Failed to unpack test data");

        assert_eq!(unpacked.day, Day::Mon);
        assert_eq!(unpacked.subject_id, 0);
        assert_eq!(unpacked.repeat, Repeat::Both);
        assert_eq!(unpacked.slot, Slot::IV);
    }

    #[test]
    fn assigned_unpacks() {
        let packed = vec!["1".into(), "2".into()];

        let unpacked = Assigned::unpack(packed).expect("Failed to unpack test data");

        assert_eq!(unpacked.meeting_id, 1);
        assert_eq!(unpacked.subject_id, 2);
    }
}

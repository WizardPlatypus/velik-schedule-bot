use crate::data::Slot;
use teloxide::utils::markdown as md;

pub struct Subject {
    slot: Slot,
    title: String,
    meetings: Vec<Meeting>,
}

impl Subject {
    pub fn new(slot: Slot, title: String, meetings: Vec<Meeting>) -> Subject {
        Subject {
            slot,
            title,
            meetings,
        }
    }
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Slot::*;
        let slot = emojis::get_by_shortcode(match self.slot {
            I => "one",
            II => "two",
            III => "three",
            IV => "four",
        })
        .unwrap();
        write!(f, "{} {}", slot, &self.title)?;
        for m in &self.meetings {
            write!(f, "\n:teacher: {}", m)?;
        }
        Ok(())
    }
}

pub struct Meeting {
    name: String,
    url: Option<String>,
}

impl Meeting {
    pub fn new(name: String, url: Option<String>) -> Meeting {
        Meeting { name, url }
    }
}

impl std::fmt::Display for Meeting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = &self.url {
            write!(f, "{}", md::link(value, &self.name))
        } else {
            write!(f, "{}", &self.name)
        }
    }
}

use event::Event;
mod apply;
mod get_length_ratio;
mod helpers;

pub trait Apply {
    fn apply(&self, events: Vec<Event>) -> Vec<Event>;
}

pub trait GetLengthRatio {
    fn get_length_ratio(&self) -> f32;
}


#[cfg(test)]
mod test;

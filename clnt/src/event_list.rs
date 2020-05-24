use std::{collections::VecDeque, time::Duration};

use instant::Instant;

use quicksilver::{
    geom::Vector,
    graphics::{Color, FontRenderer, Graphics},
};

use comn::{DeathReason, Event};

#[derive(Debug, Clone)]
pub struct Config {
    pub num_lines: usize,
    pub max_age: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_lines: 4,
            max_age: Duration::from_secs(10),
        }
    }
}

pub struct EventList {
    config: Config,
    events: VecDeque<(Instant, Event)>,
}

impl EventList {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            events: VecDeque::new(),
        }
    }

    pub fn push(&mut self, event: Event) {
        self.events.push_back((Instant::now(), event));
    }

    pub fn render(
        &mut self,
        gfx: &mut Graphics,
        font: &mut FontRenderer,
        mut pos: Vector,
    ) -> quicksilver::Result<()> {
        // Remove events that are too old.
        let now = Instant::now();
        while let Some((oldest_time, _)) = self.events.front() {
            if now.duration_since(*oldest_time) > self.config.max_age {
                self.events.pop_front();
            } else {
                break;
            }
        }

        // Display events.
        for (_, event) in self.events.iter() {
            if let Some(string) = Self::event_to_string(event) {
                font.draw(gfx, &string, Color::BLACK, pos)?;
            }
            pos.y += 12.0;
        }

        Ok(())
    }

    pub fn event_to_string(event: &Event) -> Option<String> {
        // TODO: Use player names
        match event {
            Event::PlayerDied { player_id, reason } => Some(match reason {
                DeathReason::ShotByPlayer(other_player_id) => {
                    format!("{} got shot by {}", player_id.0, other_player_id.0)
                }
                DeathReason::TouchedTheDanger => format!("{} touched the danger", player_id.0),
            }),
            _ => None,
        }
    }
}
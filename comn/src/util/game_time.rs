use std::collections::VecDeque;

use instant::Instant;

use crate::{util::stats, GameTime};

#[derive(Debug, Clone)]
pub struct GameTimeEstimation {
    recv_period: GameTime,
    recv_times: VecDeque<(Instant, GameTime)>,
}

impl GameTimeEstimation {
    pub fn new(recv_period: GameTime) -> Self {
        Self {
            recv_period,
            recv_times: VecDeque::new(),
        }
    }

    pub fn record_tick(&mut self, recv_time: Instant, game_time: GameTime) {
        if let Some((last_recv_time, last_game_time)) = self.recv_times.back() {
            if game_time < *last_game_time {
                // Received packages out of order, just ignore
                return;
            }

            assert!(recv_time >= *last_recv_time);
        }

        self.recv_times.push_back((recv_time, game_time));

        if self.recv_times.len() > 1000 {
            self.recv_times.pop_front();
        }
    }

    pub fn shifted_recv_times(&self) -> Option<impl Iterator<Item = (f32, GameTime)> + '_> {
        self.recv_times
            .front()
            .copied()
            .map(|(first_recv_time, first_game_time)| {
                self.recv_times.iter().map(move |(recv_time, game_time)| {
                    let delta_recv_time = recv_time.duration_since(first_recv_time).as_secs_f32();
                    let delta_game_time = game_time - first_game_time;

                    (delta_recv_time, delta_game_time)
                })
            })
    }

    pub fn linear_regression(&self) -> Option<stats::LinearRegression> {
        self.shifted_recv_times()
            .map(|samples| stats::linear_regression_with_beta(self.recv_period, samples))
    }

    pub fn recv_delay_std_dev(&self) -> Option<f32> {
        /*self.shifted_recv_times().map(|samples| {
            let samples: Vec<(f32, f32)> = samples.collect();
            let line = stats::linear_regression_with_beta(1.0, samples.iter().copied());

            let recv_delay = samples
                .iter()
                .map(|(delta_time, delta_game_time)| line.eval(*delta_time) - delta_game_time);

            stats::std_dev(recv_delay)
        })*/

        if !self.recv_times.is_empty() {
            Some(stats::std_dev(
                self.recv_times
                    .iter()
                    .zip(self.recv_times.iter().skip(1))
                    .map(|((recv_a, _), (recv_b, _))| recv_b.duration_since(*recv_a).as_secs_f32()),
            ))
        } else {
            None
        }
    }

    pub fn has_started(&self) -> bool {
        !self.recv_times.is_empty()
    }

    pub fn estimate(&self, now: Instant) -> Option<GameTime> {
        self.recv_times
            .front()
            .and_then(|(first_recv_time, first_game_time)| {
                self.shifted_recv_times().map(|samples| {
                    let line = stats::linear_regression_with_beta(1.0, samples);
                    let delta_recv_time = now.duration_since(*first_recv_time).as_secs_f32();
                    let delta_game_time = line.eval(delta_recv_time);

                    first_game_time + delta_game_time
                })
            })
    }
}

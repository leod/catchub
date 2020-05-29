use std::collections::{BTreeMap, BTreeSet};

use crate::entities::Bullet;
use crate::{
    geom::AaRect, DeathReason, Entity, EntityId, Event, Game, GameError, GameResult, Input,
    PlayerEntity, PlayerId, TickNum, Vector,
};

pub const PLAYER_MOVE_SPEED: f32 = 300.0;
pub const PLAYER_SIT_W: f32 = 50.0;
pub const PLAYER_SIT_L: f32 = 50.0;
pub const PLAYER_MOVE_W: f32 = 70.0;
pub const PLAYER_MOVE_L: f32 = 35.714;
pub const PLAYER_SHOOT_PERIOD: f32 = 0.3;
pub const BULLET_MOVE_SPEED: f32 = 400.0;

#[derive(Clone, Debug, Default)]
pub struct RunContext {
    pub events: Vec<Event>,
    pub new_entities: Vec<Entity>,
    pub removed_entities: BTreeSet<EntityId>,
    pub killed_players: BTreeMap<PlayerId, DeathReason>,
}

impl Game {
    pub fn run_tick(&mut self, context: &mut RunContext) -> GameResult<()> {
        let time = self.current_game_time();

        for (entity_id, entity) in self.entities.iter() {
            match entity {
                Entity::Bullet(bullet) => {
                    if !self.settings.aa_rect().contains_point(bullet.pos(time)) {
                        context.removed_entities.insert(*entity_id);
                        continue;
                    }

                    for (entity_id_b, entity_b) in self.entities.iter() {
                        if *entity_id == *entity_id_b {
                            continue;
                        }

                        match entity_b {
                            Entity::DangerGuy(danger_guy) => {
                                if danger_guy.aa_rect(time).contains_point(bullet.pos(time)) {
                                    context.removed_entities.insert(*entity_id);
                                }
                            }
                            Entity::Player(player) if player.owner != bullet.owner => {
                                // TODO: Player geometry
                                let aa_rect = AaRect::new_center(
                                    player.pos,
                                    Vector::new(PLAYER_SIT_W, PLAYER_SIT_L),
                                );

                                if aa_rect.contains_point(bullet.pos(time)) {
                                    context.removed_entities.insert(*entity_id);
                                    context.killed_players.insert(
                                        player.owner,
                                        DeathReason::ShotByPlayer(bullet.owner),
                                    );
                                }
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            }
        }

        Ok(())
    }

    pub fn run_player_input(
        &mut self,
        player_id: PlayerId,
        input: &Input,
        input_tick: Option<TickNum>,
        context: &mut RunContext,
    ) -> GameResult<()> {
        let delta_s = self.settings.tick_period();
        let time = self.current_game_time();
        let input_time = input_tick
            .map(|num| self.tick_game_time(num))
            .unwrap_or(time);
        let map_size = self.settings.size;

        if let Some((_entity_id, player_entity)) = self.get_player_entity_mut(player_id)? {
            let mut delta = Vector::new(0.0, 0.0);

            if input.move_left {
                delta.x -= 1.0;
            }
            if input.move_right {
                delta.x += 1.0;
            }
            if input.move_up {
                delta.y -= 1.0;
            }
            if input.move_down {
                delta.y += 1.0;
            }

            if delta.norm() > 0.0 {
                player_entity.pos += delta.normalize() * PLAYER_MOVE_SPEED * delta_s;
                player_entity.angle = Some(delta.y.atan2(delta.x));
            } else {
                player_entity.angle = None;
            }

            player_entity.pos.x = player_entity
                .pos
                .x
                .min(map_size.x - PLAYER_SIT_W / 2.0)
                .max(PLAYER_SIT_W / 2.0);
            player_entity.pos.y = player_entity
                .pos
                .y
                .min(map_size.y - PLAYER_SIT_W / 2.0)
                .max(PLAYER_SIT_W / 2.0);

            if delta.norm() > 0.0
                && input.use_item
                && input_time - player_entity.last_shot_time.unwrap_or(-1000.0)
                    >= PLAYER_SHOOT_PERIOD
            {
                //log::info!("last shot at {:?}, shooting now {:?}", player_entity.last_shot_time, input_time);
                player_entity.last_shot_time = Some(input_time);
                context.new_entities.push(Entity::Bullet(Bullet {
                    owner: player_id,
                    start_time: input_time,
                    start_pos: player_entity.pos,
                    vel: delta.normalize() * BULLET_MOVE_SPEED,
                }));
            }

            let pos = player_entity.pos;
            for (_entity_id, entity) in self.entities.iter() {
                match entity {
                    Entity::DangerGuy(danger_guy) => {
                        // TODO: Player geometry
                        if danger_guy.aa_rect(input_time).contains_point(pos) {
                            context
                                .killed_players
                                .insert(player_id, DeathReason::TouchedTheDanger);
                        }
                    }
                    _ => (),
                }
            }
        }

        Ok(())
    }

    pub fn get_entity(&mut self, entity_id: EntityId) -> GameResult<&Entity> {
        self.entities
            .get(&entity_id)
            .ok_or_else(|| GameError::InvalidEntityId(entity_id))
    }

    pub fn get_player_entity(
        &self,
        player_id: PlayerId,
    ) -> GameResult<Option<(EntityId, &PlayerEntity)>> {
        Ok(self
            .entities
            .iter()
            .filter_map(|(&id, e)| {
                if let Entity::Player(ref e) = e {
                    if e.owner == player_id {
                        Some((id, e))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .next())
    }

    pub fn get_player_entity_mut(
        &mut self,
        player_id: PlayerId,
    ) -> GameResult<Option<(EntityId, &mut PlayerEntity)>> {
        Ok(self
            .entities
            .iter_mut()
            .filter_map(|(&id, e)| {
                if let Entity::Player(ref mut e) = e {
                    if e.owner == player_id {
                        Some((id, e))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .next())
    }
}

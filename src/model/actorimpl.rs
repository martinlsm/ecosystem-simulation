use std::cmp::{max, min};
use std::f64::consts::PI;

use crate::model::actor::*;

#[derive(Debug)]
pub struct ActorImpl {
    pos: (f32, f32),

    curr_speed: f32,
    desired_speed: f32,

    curr_angle: f32,
    desired_angle: f32,

    health: f32,
    endurance: f32,
}

impl ActorImpl {
    const MAX_SPEED: f32 = 100.0; // XXX
    const ACCELERATION_FACTOR: f32 = 600.0;
    const TURN_RATE: f32 = (1.0 / 2.0) * (2.0 * (PI as f32));
    const MAX_HEALTH: f32 = 100.0;
    const MAX_ENDURANCE: f32 = 100.0;
    const ENDURANCE_DEPLETION_RATE: f32 = 5.0;
    const ENDURANCE_RECHARGE_RATE: f32 = 15.0;

    pub fn new(starting_pos: (f32, f32)) -> Self {
        ActorImpl {
            pos: starting_pos,

            curr_speed: 0.0,
            desired_speed: 0.0,

            curr_angle: 0.0,
            desired_angle: 0.0,

            health: ActorImpl::MAX_HEALTH,
            endurance: ActorImpl::MAX_ENDURANCE,
        }
    }
}

impl Actor for ActorImpl {
    fn update(&mut self, dt: f32) {
        // Update speed.
        if self.curr_speed < self.desired_speed {
            self.curr_speed += ActorImpl::ACCELERATION_FACTOR * dt;
        } else if self.curr_speed > self.desired_speed {
            self.curr_speed -= ActorImpl::ACCELERATION_FACTOR * dt;
        }
        self.curr_speed = self.curr_speed.clamp(0.0, ActorImpl::MAX_SPEED);

        // Update endurance.
        if self.curr_speed > 0.0 {
            self.endurance -= (self.curr_speed / ActorImpl::MAX_SPEED) * ActorImpl::ENDURANCE_DEPLETION_RATE * dt;
        } else {
            self.endurance += ActorImpl::ENDURANCE_RECHARGE_RATE * dt;
        }
        self.endurance = self.endurance.clamp(0.0, ActorImpl::MAX_ENDURANCE);

        // Update angle.
        let turn_step = ActorImpl::TURN_RATE * dt;
        if (self.desired_angle - self.curr_angle).abs() < turn_step {
            self.curr_angle = self.desired_angle;
        } else if self.curr_angle < self.desired_angle {
            self.curr_angle += turn_step;
        } else {
            self.curr_angle -= turn_step;
        }

        // Update pos.
        let new_pos_x = self.pos.0 + self.curr_angle.cos() * self.curr_speed * dt;
        let new_pos_y = self.pos.1 + self.curr_angle.sin() * self.curr_speed * dt;
        self.pos = (new_pos_x, new_pos_y);
    }

    fn do_action(&mut self, action: Action) -> Result<(), &'static str> { 
        match action {
            Action::Forward => self.desired_speed = ActorImpl::MAX_SPEED,
            Action::Stop => self.desired_speed = 0.0,
            Action::Turn(rad) => self.desired_angle += rad,
        }

        Ok(())
    }

    fn get_pos(&self) -> (f32, f32) {
        self.pos
    }
}

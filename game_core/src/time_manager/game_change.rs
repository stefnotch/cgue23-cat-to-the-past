use std::collections::VecDeque;

use crate::events::NextLevel;
use bevy_ecs::{
    prelude::{not, EventReader},
    schedule::{IntoSystemConfig, Schedule},
    system::{ResMut, Resource},
    world::World,
};

use crate::application::AppStage;

use super::{is_rewinding, level_time::LevelTime, TimeManager};

pub trait GameChange
where
    Self: Sync + Send,
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterpolationType {
    None,
    Linear,
}

pub struct GameChangeInterpolation<'history, T>
where
    T: GameChange,
{
    pub from: &'history GameChanges<T>,
    pub to: &'history GameChanges<T>,
    pub factor: f32,
}

/// All game changes in one frame
/// Multiple commands, because we have multiple entities
pub struct GameChanges<T>
where
    T: GameChange,
{
    timestamp: LevelTime,
    pub commands: Vec<T>,
}

/// To limit the size of this, we could either
/// - have a countdown for every level
/// - only save actual changes, so when the user is AFK, we don't save anything
/// - have a max size and remove the oldest commands,
///   this is especially useful when it's always possible to restart the level simply by walking back to the beginning
#[derive(Resource)]
pub struct GameChangeHistory<T>
where
    T: GameChange,
{
    history: VecDeque<GameChanges<T>>,
}

impl<T> GameChangeHistory<T>
where
    T: GameChange,
{
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
        }
    }

    pub fn add_command(&mut self, timestamp: LevelTime, command: T) {
        if let Some(last) = self.history.back_mut() {
            if last.timestamp == timestamp {
                last.commands.push(command);
                return;
            }
        }
        self.history.push_back(GameChanges {
            timestamp,
            commands: vec![command],
        });
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Returns the commands that need to be applied to the game state
    pub fn take_commands_to_apply(
        &mut self,
        time_manager: &TimeManager,
        with_interpolation: InterpolationType,
    ) -> (Vec<GameChanges<T>>, Option<GameChangeInterpolation<T>>) {
        let mut commands = Vec::new();
        loop {
            if self.history.len() <= 1 {
                // If there's only one element, we can't really rewind time any further
                break;
            }

            let top = self.history.back().unwrap();

            // If we're further back in the past
            if time_manager.level_time < top.timestamp {
                // We can pop the top and apply it
                let top = self.history.pop_back().unwrap();
                commands.push(top);
            } else {
                // Nothing to do
                break;
            }
        }

        let interpolation = if with_interpolation == InterpolationType::Linear
            && commands
                .last()
                .map(|v| self.can_interpolate(v, time_manager))
                == Some(true)
        {
            // We add it back to the history
            let top = commands.pop().unwrap();
            self.history.push_back(top);
            let top = self.history.back().unwrap();

            // And return the desired interpolation data
            let previous = self.history.get(self.history.len() - 2).unwrap();
            assert!(previous.timestamp <= top.timestamp);
            let factor = previous
                .timestamp
                .inverse_lerp(&top.timestamp, time_manager.level_time);
            Some(GameChangeInterpolation {
                from: previous,
                to: top,
                factor,
            })
        } else {
            None
        };

        (commands, interpolation)
    }

    fn can_interpolate(&self, top: &GameChanges<T>, time_manager: &TimeManager) -> bool {
        if !time_manager.is_interpolating() {
            return false;
        }
        if self.history.len() < 1 {
            return false;
        }

        return time_manager.level_time < top.timestamp;
    }
}

impl<T> GameChangeHistory<T>
where
    T: GameChange + 'static,
{
    pub fn setup_systems<TrackerParams, RewinderParams>(
        self,
        world: &mut World,
        schedule: &mut Schedule,
        tracker_system: impl IntoSystemConfig<TrackerParams>,
        rewinder_system: impl IntoSystemConfig<RewinderParams>,
    ) where
        T: GameChange,
    {
        world.insert_resource(self);

        schedule.add_system(
            tracker_system
                .in_set(AppStage::Render)
                .run_if(not(is_rewinding)),
        );
        schedule.add_system(
            rewinder_system
                .in_set(AppStage::EventUpdate)
                .run_if(is_rewinding),
        );
        schedule.add_system(clear_on_next_level::<T>.in_set(AppStage::StartFrame));
    }
}

fn clear_on_next_level<T>(
    mut history: ResMut<GameChangeHistory<T>>,
    mut next_level: EventReader<NextLevel>,
) where
    T: GameChange + 'static,
{
    if next_level.iter().next().is_some() {
        history.clear();
    }
}

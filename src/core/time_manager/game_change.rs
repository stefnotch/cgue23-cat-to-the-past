use std::collections::VecDeque;

use bevy_ecs::{
    prelude::{not, EventReader},
    schedule::IntoSystemConfig,
    system::{ResMut, Resource},
    world::EntityMut,
};

use crate::core::{
    application::{AppStage, ApplicationBuilder},
    events::NextLevel,
};

use super::{is_rewinding, level_time::LevelTime, TimeManager};

pub trait GameChange
where
    Self: Sync + Send,
{
}

/// All game changes in one frame
/// Multiple commands, because we have multiple entities
pub(super) struct GameChanges<T>
where
    T: GameChange,
{
    timestamp: LevelTime,
    commands: Vec<T>,
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

pub(super) enum StateLookup {
    Nearest(LevelTime),
    Interpolated(LevelTime),
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

    pub fn get_commands_to_apply(&mut self, time_manager: &TimeManager) -> Vec<GameChanges<T>> {
        let mut commands = Vec::new();
        loop {
            if self.history.len() < 3 {
                // If there's only one element, we can't really rewind time any further
                // If there are only two elements, we don't have to apply any commands, instead we interpolate between them
                break;
            }

            let _top = self.history.get(self.history.len() - 1).unwrap();
            let previous = self.history.get(self.history.len() - 2).unwrap();

            // If we're further back in the past
            if time_manager.level_time < previous.timestamp {
                // We can pop the top and apply it
                let top = self.history.pop_back().unwrap();
                commands.push(top);
            } else {
                // Nothing to do
                break;
            }
        }

        commands
    }

    pub fn get_commands_to_interpolate(
        &self,
        time_manager: &TimeManager,
    ) -> Option<(&GameChanges<T>, &GameChanges<T>, f32)> {
        if self.history.len() < 2 {
            return None;
        }

        let top = self.history.get(self.history.len() - 1).unwrap();
        let previous = self.history.get(self.history.len() - 2).unwrap();

        if time_manager.level_time < top.timestamp {
            let factor = previous
                .timestamp
                .inverse_lerp(&top.timestamp, time_manager.level_time);
            Some((previous, top, factor))
        } else {
            None
        }
    }
}

impl ApplicationBuilder {
    pub fn with_game_change_history<T, TrackerParams, RewinderParams>(
        self,
        tracker_system: impl IntoSystemConfig<TrackerParams>,
        rewinder_system: impl IntoSystemConfig<RewinderParams>,
    ) -> Self
    where
        T: GameChange,
    {
        self.with_resource(GameChangeHistory::<T>::new())
            .with_system(
                tracker_system
                    .in_set(AppStage::Render)
                    .run_if(not(is_rewinding)),
            )
            .with_system(
                rewinder_system
                    .in_set(AppStage::EventUpdate)
                    .run_if(is_rewinding),
            )
            .with_system(clear_on_next_level::<T>.in_set(AppStage::StartFrame))
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

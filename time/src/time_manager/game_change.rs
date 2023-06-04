use std::collections::VecDeque;

use crate::events::NextLevel;
use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::{
    prelude::{not, EventReader},
    schedule::{IntoSystemConfig, IntoSystemSetConfig, SystemConfig, SystemSet},
    system::{Res, ResMut, Resource},
};

use super::{is_rewinding, level_time::LevelTime, TimeManager, TimeManagerPluginSet};

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

/// Systems change object values.
/// Time rewinding restores the state of an object before a system acts on it.
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
    is_rewinding: bool,
    level_time: LevelTime,
    history: VecDeque<GameChanges<T>>,
    rewinder_commands: Vec<T>,
}

impl<T> GameChangeHistory<T>
where
    T: GameChange,
{
    pub fn new() -> Self {
        Self {
            is_rewinding: false,
            level_time: LevelTime::zero(),
            history: VecDeque::new(),
            rewinder_commands: Vec::new(),
        }
    }

    fn update_with_time(&mut self, time_manager: &TimeManager) {
        self.is_rewinding = time_manager.is_rewinding();
        self.level_time = time_manager.level_time;
        if !self.is_rewinding {
            self.rewinder_commands.clear();
        }
    }

    pub fn add_command(&mut self, command: T) {
        assert!(!self.is_rewinding, "Cannot add commands while rewinding");

        if let Some(last) = self.history.back_mut() {
            if last.timestamp == self.level_time {
                last.commands.push(command);
                return;
            }
        }

        // This logic avoids adding commands to the history that are not needed
        self.history.push_back(GameChanges {
            timestamp: self.level_time,
            commands: vec![command],
        });
    }

    fn clear(&mut self) {
        self.history.clear();
        self.history.push_back(GameChanges {
            timestamp: LevelTime::zero(),
            commands: Vec::new(),
        });
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
                .inverse_lerp(&top.timestamp, time_manager.level_time)
                as f32;
            Some(GameChangeInterpolation {
                from: previous,
                to: top,
                factor,
            })
        } else {
            None
        };

        if self.rewinder_commands.len() > 0 {
            commands.insert(
                0,
                GameChanges {
                    timestamp: time_manager.level_time,
                    commands: std::mem::replace(&mut self.rewinder_commands, Vec::new()),
                },
            );
        }
        (commands, interpolation)
    }

    fn can_interpolate(&self, top: &GameChanges<T>, time_manager: &TimeManager) -> bool {
        if !time_manager.is_interpolating() {
            return false;
        }
        if self.history.is_empty() {
            return false;
        }

        return time_manager.level_time < top.timestamp;
    }
}

fn read_timestamp<T>(time_manager: Res<TimeManager>, mut history: ResMut<GameChangeHistory<T>>)
where
    T: GameChange + 'static,
{
    history.update_with_time(&time_manager);
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

#[derive(SystemSet)]
enum GameChangeHistoryPluginSet<T> {
    UpdateInfo,
    Track,
    Rewind,
    // Well that's not very elegant
    _Marker(std::convert::Infallible, std::marker::PhantomData<T>),
}

impl<T> std::fmt::Debug for GameChangeHistoryPluginSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UpdateInfo => write!(f, "GameChangeHistoryPluginSet::UpdateInfo"),
            Self::Track => write!(f, "GameChangeHistoryPluginSet::Track"),
            Self::Rewind => write!(f, "GameChangeHistoryPluginSet::Rewind"),
            Self::_Marker(_, _) => {
                write!(f, "GameChangeHistoryPluginSet::_Impossible")
            }
        }
    }
}
impl<T> Clone for GameChangeHistoryPluginSet<T> {
    fn clone(&self) -> Self {
        match self {
            Self::UpdateInfo => Self::UpdateInfo,
            Self::Track => Self::Track,
            Self::Rewind => Self::Rewind,
            Self::_Marker(_arg0, _arg1) => panic!("d"),
        }
    }
}
impl<T> PartialEq for GameChangeHistoryPluginSet<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UpdateInfo, Self::UpdateInfo) => true,
            (Self::Track, Self::Track) => true,
            (Self::Rewind, Self::Rewind) => true,
            (Self::_Marker(_arg0, _arg1), Self::_Marker(_arg0_other, _arg1_other)) => {
                panic!("e")
            }
            _ => false,
        }
    }
}

impl<T> Eq for GameChangeHistoryPluginSet<T> {}

impl<T> std::hash::Hash for GameChangeHistoryPluginSet<T>
where
    T: 'static,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::UpdateInfo => std::any::TypeId::of::<T>().hash(state),
            Self::Track => std::any::TypeId::of::<T>().hash(state),
            Self::Rewind => std::any::TypeId::of::<T>().hash(state),
            Self::_Marker(_arg0, _arg1) => panic!("a"),
        }
    }
}

pub struct GameChangeHistoryPlugin<T>
where
    T: GameChange + 'static,
{
    systems: Vec<SystemConfig>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> GameChangeHistoryPlugin<T>
where
    T: GameChange + 'static,
{
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_tracker<Params>(self, system: impl IntoSystemConfig<Params>) -> Self {
        let system_config = system.in_set(GameChangeHistoryPluginSet::<T>::Track);

        let mut systems = self.systems;
        systems.push(system_config);

        Self {
            systems,
            _marker: self._marker,
        }
    }

    pub fn with_rewinder<Params>(self, system: impl IntoSystemConfig<Params>) -> Self {
        let system_config = system.in_set(GameChangeHistoryPluginSet::<T>::Rewind);

        let mut systems = self.systems;
        systems.push(system_config);

        Self {
            systems,
            _marker: self._marker,
        }
    }
}

impl<T> Plugin for GameChangeHistoryPlugin<T>
where
    T: GameChange + 'static,
    GameChangeHistoryPluginSet<T>: SystemSet, // Wait, it accepts this?
{
    fn build(&mut self, app: &mut PluginAppAccess) {
        let systems = self.systems.drain(..);
        for system in systems {
            app.with_system(system);
        }

        app //
            .with_resource(GameChangeHistory::<T>::new())
            .with_set(GameChangeHistoryPluginSet::<T>::Track.run_if(not(is_rewinding)))
            .with_set(GameChangeHistoryPluginSet::<T>::Rewind.run_if(is_rewinding))
            .with_set(
                TimeManagerPluginSet::StartFrame
                    .before(GameChangeHistoryPluginSet::<T>::UpdateInfo),
            )
            .with_set(
                GameChangeHistoryPluginSet::<T>::UpdateInfo
                    .before(GameChangeHistoryPluginSet::<T>::Track),
            )
            .with_set(
                GameChangeHistoryPluginSet::<T>::UpdateInfo
                    .before(GameChangeHistoryPluginSet::<T>::Rewind),
            )
            .with_set(
                GameChangeHistoryPluginSet::<T>::Track
                    .ambiguous_with(GameChangeHistoryPluginSet::<T>::Rewind),
            )
            .with_system(
                clear_on_next_level::<T>
                    .in_set(GameChangeHistoryPluginSet::<T>::UpdateInfo)
                    .before(read_timestamp::<T>),
            )
            .with_system(read_timestamp::<T>.in_set(GameChangeHistoryPluginSet::<T>::UpdateInfo));
    }
}

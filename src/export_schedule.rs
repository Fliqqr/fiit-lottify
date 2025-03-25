use bevy::{app::MainScheduleOrder, ecs::schedule::*, prelude::*};

/// Independent [`Schedule`] for stepping systems.
///
/// The stepping systems must run in their own schedule to be able to inspect
/// all the other schedules in the [`App`].  This is because the currently
/// executing schedule is removed from the [`Schedules`] resource while it is
/// being run.
#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
struct ExportSchedule;

/// Plugin to add a stepping UI to an example
#[derive(Default)]
pub struct SteppingPlugin {
    schedule_labels: Vec<InternedScheduleLabel>,
    top: Val,
    left: Val,
}

impl SteppingPlugin {
    /// add a schedule to be stepped when stepping is enabled
    pub fn add_schedule(mut self, label: impl ScheduleLabel) -> SteppingPlugin {
        self.schedule_labels.push(label.intern());
        self
    }
}

impl Plugin for SteppingPlugin {
    fn build(&self, app: &mut App) {
        println!("Building SteppingPluin");

        // if cfg!(not(feature = "bevy_debug_stepping")) {
        //     return;
        // }

        // create and insert our debug schedule into the main schedule order.
        // We need an independent schedule so we have access to all other
        // schedules through the `Stepping` resource
        app.init_schedule(ExportSchedule);
        let mut order = app.world_mut().resource_mut::<MainScheduleOrder>();
        order.insert_after(Update, ExportSchedule);

        // create our stepping resource
        let mut stepping = Stepping::new();
        for label in &self.schedule_labels {
            println!("{:?}", label);
            stepping.add_schedule(*label);
        }
        app.insert_resource(stepping);

        // add our startup & stepping systems
        app.insert_resource(State {
            ui_top: self.top,
            ui_left: self.left,
            systems: Vec::new(),
        });
        // .add_systems(ExportSchedule, (handle_input,).chain());
    }
}

/// Struct for maintaining stepping state
#[derive(Resource, Debug)]
struct State {
    // vector of schedule/node id -> text index offset
    systems: Vec<(InternedScheduleLabel, NodeId, usize)>,

    // ui positioning
    ui_top: Val,
    ui_left: Val,
}

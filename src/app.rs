use bevy::prelude::*;
use bevy::app::App;

#[derive(Resource)]
struct GreetTimer(Timer);

pub fn a() {
    App::new()
        .insert_resource(GreetTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_plugins(DefaultPlugins)
        .add_event()
        .add_systems(Update, foo)
        .run();
}

fn foo(time: ResMut<Time<Fixed>>, timer: Res<GreetTimer>) {
    //time.tic
    println!("{:?}", time.delta());
}

struct M;

impl Plugin for M {
    fn build(&self, app: &mut App) {
        
    }
}
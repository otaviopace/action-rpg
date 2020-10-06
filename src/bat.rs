use crate::extensions::NodeExt;
use crate::player::Player;
use gdnative::api::Area2D;
use gdnative::prelude::{KinematicBody2D, NativeClass, Ref, Vector2, Vector2Godot};

#[derive(NativeClass)]
#[inherit(KinematicBody2D)]
#[derive(Default)]
pub struct Bat {
    knockback: Vector2,
}

#[gdnative::methods]
impl Bat {
    fn new(_owner: &KinematicBody2D) -> Self {
        Self::default()
    }

    #[export]
    fn _process(&mut self, _owner: &KinematicBody2D, delta: f32) {
        self.knockback = self.knockback.move_towards(Vector2::zero(), 200.0 * delta);
    }

    #[export]
    fn _physics_process(&mut self, owner: &KinematicBody2D, _delta: f32) {
        self.knockback =
            owner.move_and_slide(self.knockback, Vector2::zero(), false, 4, 0.785398, true);
    }

    #[export]
    #[allow(non_snake_case)]
    fn _on_Hurtbox_area_entered(&mut self, owner: &KinematicBody2D, _x: Ref<Area2D>) {
        let player_node = unsafe { owner.get_typed_node::<KinematicBody2D, _>("../Player") };

        let instance = player_node.cast_instance::<Player>().unwrap();

        let _ = instance.map(|player, _| {
            self.knockback = player.knockback_vector * 120.0;
        });
    }
}

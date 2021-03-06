use crate::extensions::{NodeExt, Vector2Ext};
use crate::sword_hitbox::SwordHitbox;
use gdnative::api::{AnimationNodeStateMachinePlayback, AnimationTree, Area2D};
use gdnative::prelude::{Input, KinematicBody2D, NativeClass, TRef, Vector2, Vector2Godot};

const ACCELERATION: f32 = 500.0;
const MAX_SPEED: f32 = 80.0;
const ROLL_SPEED: f32 = 120.0;
const FRICTION: f32 = 500.0;

#[derive(NativeClass)]
#[inherit(KinematicBody2D)]
#[derive(Default)]
pub struct Player {
    velocity: Vector2,
    state: State,
    roll_vector: Vector2,
}

enum State {
    Move,
    Attack,
    Roll,
}

impl Default for State {
    fn default() -> Self {
        Self::Move
    }
}

#[gdnative::methods]
impl Player {
    fn new(_owner: &KinematicBody2D) -> Self {
        Player {
            roll_vector: Vector2::down(),
            ..Default::default()
        }
    }

    #[export]
    fn _process(&mut self, owner: &KinematicBody2D, delta: f32) {
        let animation_tree = unsafe { owner.get_typed_node::<AnimationTree, _>("AnimationTree") };
        let playback_prop = animation_tree
            .get("parameters/playback")
            .try_to_object()
            .unwrap();
        let animation_state: TRef<AnimationNodeStateMachinePlayback> =
            unsafe { playback_prop.assume_safe() };

        let sword_hitbox_node =
            unsafe { owner.get_typed_node::<Area2D, _>("HitboxPivot/SwordHitbox") };

        let instance = sword_hitbox_node.cast_instance::<SwordHitbox>().unwrap();

        let input_singleton = Input::godot_singleton();

        match self.state {
            State::Move => {
                let input_vector = self.get_movement_input(input_singleton);

                self.animate_on_input(&animation_tree, &animation_state, input_vector);

                let _ = instance.map_mut(|sword_hitbox, _| {
                    self.move_on_input(input_vector, delta, sword_hitbox);
                });

                self.handle_attack_input(input_singleton);
                self.handle_roll_input(input_singleton);
            }
            State::Attack => {
                self.animate_attack(&animation_state);
            }
            State::Roll => {
                self.roll();
                self.animate_roll(&animation_state);
            }
        };
    }

    #[export]
    fn _physics_process(&mut self, owner: &KinematicBody2D, _delta: f32) {
        match self.state {
            State::Move => {
                self.velocity =
                    owner.move_and_slide(self.velocity, Vector2::zero(), false, 4, 0.785398, true);
            }
            State::Attack => {
                self.velocity = Vector2::zero();
            }
            State::Roll => {
                self.velocity =
                    owner.move_and_slide(self.velocity, Vector2::zero(), false, 4, 0.785398, true);
            }
        }
    }

    fn get_movement_input(&self, input: &Input) -> Vector2 {
        let right_strength = input.get_action_strength("ui_right");
        let left_strength = input.get_action_strength("ui_left");
        let down_strength = input.get_action_strength("ui_down");
        let up_strength = input.get_action_strength("ui_up");

        let mut input_vector = Vector2::zero();

        input_vector.x = (right_strength - left_strength) as f32;
        input_vector.y = (down_strength - up_strength) as f32;

        input_vector.try_normalize().unwrap_or(input_vector)
    }

    fn animate_on_input(
        &self,
        animation_tree: &AnimationTree,
        animation_state: &AnimationNodeStateMachinePlayback,
        input_vector: Vector2,
    ) {
        if input_vector != Vector2::zero() {
            animation_tree.set("parameters/Idle/blend_position", input_vector);
            animation_tree.set("parameters/Run/blend_position", input_vector);
            animation_tree.set("parameters/Attack/blend_position", input_vector);
            animation_tree.set("parameters/Roll/blend_position", input_vector);

            animation_state.travel("Run");
        } else {
            animation_state.travel("Idle");
        }
    }

    fn move_on_input(&mut self, input_vector: Vector2, delta: f32, sword_hitbox: &mut SwordHitbox) {
        if input_vector != Vector2::zero() {
            self.roll_vector = input_vector;
            sword_hitbox.knockback_vector = input_vector;

            self.velocity = self
                .velocity
                .move_towards(input_vector * MAX_SPEED, ACCELERATION * delta);
        } else {
            self.velocity = self
                .velocity
                .move_towards(Vector2::zero(), FRICTION * delta);
        }
    }

    fn roll(&mut self) {
        self.velocity = self.roll_vector * ROLL_SPEED;
    }

    fn handle_attack_input(&mut self, input: &Input) {
        if input.is_action_just_pressed("attack") {
            self.state = State::Attack;
        }
    }

    fn handle_roll_input(&mut self, input: &Input) {
        if input.is_action_just_pressed("roll") {
            self.state = State::Roll;
        }
    }

    fn animate_attack(&mut self, animation_state: &AnimationNodeStateMachinePlayback) {
        animation_state.travel("Attack");
    }

    fn animate_roll(&mut self, animation_state: &AnimationNodeStateMachinePlayback) {
        animation_state.travel("Roll");
    }

    #[export]
    fn attack_animation_finished(&mut self, _owner: &KinematicBody2D) {
        self.state = State::Move;
    }

    #[export]
    fn roll_animation_finished(&mut self, _owner: &KinematicBody2D) {
        self.velocity = self.velocity * 0.8;
        self.state = State::Move;
    }
}

#[test]
fn test_move_nothing() {
    let mut player = Player::default();

    player.r#move(0.0, 0.0, 0.0, 0.0, 0.6);

    assert_eq!(player.velocity, Vector2::zero());
}

#[test]
fn test_move_right() {
    let mut player = Player::default();

    player.r#move(1.0, 0.0, 0.0, 0.0, 0.6);

    assert_eq!(player.velocity, Vector2::new(1.0 * MAX_SPEED, 0.0));
}

#[test]
fn test_move_left() {
    let mut player = Player::default();

    player.r#move(0.0, 1.0, 0.0, 0.0, 0.6);

    assert_eq!(player.velocity, Vector2::new(-1.0 * MAX_SPEED, 0.0));
}

#[test]
fn test_move_down() {
    let mut player = Player::default();

    player.r#move(0.0, 0.0, 1.0, 0.0, 0.6);

    assert_eq!(player.velocity, Vector2::new(0.0, 1.0 * MAX_SPEED));
}

#[test]
fn test_move_up() {
    let mut player = Player::default();

    player.r#move(0.0, 0.0, 0.0, 1.0, 0.6);

    assert_eq!(player.velocity, Vector2::new(0.0, -1.0 * MAX_SPEED));
}

#[ignore]
#[test]
fn test_move_diagonals() {
    let mut player = Player::default();

    player.r#move(0.0, 1.0, 1.0, 0.0, 0.6);
    player.r#move(0.0, 1.0, 1.0, 0.0, 0.6);
    player.r#move(1.0, 0.0, 0.0, 1.0, 0.6);

    assert_eq!(player.velocity, Vector2::new(-4.242641, 4.242641));
}

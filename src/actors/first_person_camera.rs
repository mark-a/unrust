use engine::{Camera, Component, ComponentType, GameObject};
use uni_app::AppEvent;
use world::{Actor, Processor, World};

use math::*;
use std::sync::Arc;

bitflags! {
    struct Movement: u32 {
        const TURN_LEFT = 1;
        const TURN_RIGHT = 1 << 2;
        const FORWARD = 1 << 3;
        const BACKWARD = 1 << 4;
        const UP = 1 << 5;
        const DOWN = 1 << 6;
        const LEFT = 1 << 7;
        const RIGHT = 1 << 8;
        const MOUSE = 1 << 16;
    }
}

#[derive(Component)]
pub struct FirstPersonCamera {
    pub speed: f32,
    pub angle_speed: f32,

    pub eye: Vector3<f32>,
    pub eye_dir: Vector3<f32>,

    camera: Option<Arc<Component>>,

    state: Movement,
    handlers: Vec<(Movement, String, Box<Fn(&mut FirstPersonCamera, f64)>)>,
    mouse_pos:  Vector2<i32>,
    mouse_angle_speed: f32,

    clicked: bool,
    click_pos:  Vector2<i32>,
}

impl Processor for FirstPersonCamera {
    fn new() -> FirstPersonCamera {
        use math::*;

        let mut m = FirstPersonCamera {
            speed: 10.0,
            angle_speed: 0.5,
            state: Movement::empty(),
            handlers: Vec::new(),
            camera: None,
            eye: Vector3::new(0.0, 0.0, -3.0),
            eye_dir: Vector3::new(0.0, 0.0, 1.0).normalize(),
            mouse_pos: Vector2::new(0,0),
            mouse_angle_speed: 0.00001,
            clicked: false,
            click_pos: Vector2::new(0,0),
        };

        let up = Vector3::unit_y();

        m.add(Movement::TURN_LEFT, "KeyA", move |s, dt| {
            s.eye_dir = Quaternion::from_angle_y(Rad(s.angle_speed * dt as f32)) * s.eye_dir;
        });
        m.add(Movement::TURN_RIGHT, "KeyD", move |s, dt| {
            s.eye_dir = Quaternion::from_angle_y(Rad(-s.angle_speed * dt as f32)) * s.eye_dir
        });
        m.add(Movement::UP, "KeyE", move |s, dt| {
            s.eye = s.eye + up * s.speed * dt as f32;
        });
        m.add(Movement::DOWN, "KeyC", move |s, dt| {
            s.eye = s.eye + up * -s.speed * dt as f32;
        });
        m.add(Movement::FORWARD, "KeyW", move |s, dt| {
            s.eye = s.eye + s.eye_dir * s.speed * dt as f32;
        });
        m.add(Movement::BACKWARD, "KeyS", move |s, dt| {
            s.eye = s.eye + s.eye_dir * -s.speed * dt as f32;
        });
        m.add(Movement::LEFT, "KeyZ", move |s, dt| {
            let right = s.eye_dir.cross(up).normalize();
            s.eye = s.eye - right * s.speed * dt as f32;
        });a
        m.add(Movement::RIGHT, "KeyX", move |s, dt| {
            let right = s.eye_dir.cross(up).normalize();
            s.eye = s.eye + right * s.speed * dt as f32;
        });
        m.add(Movement::MOUSE, "", move |s, dt| {

            let delta = s.mouse_pos - s.click_pos;



/*
moveLookLocked
            if(angles.x < -M_PI)
            angles.x += M_PI * 2;
            else if(angles.x > M_PI)
            angles.x -= M_PI * 2;

            if(angles.y < -M_PI / 2)
            angles.y = -M_PI / 2;
            if(angles.y > M_PI / 2)
            angles.y = M_PI / 2;
            glm::vec3 lookat;
            lookat.x = sinf(angles.x) * cosf(angles.y);
            lookat.y = sinf(angles.y);
            lookat.z = cosf(angles.x) * cosf(angles.y);

            zAngle += xDelta*0.0025;
            while (zAngle < 0)
            zAngle += Math.PI*2;
            while (zAngle >= Math.PI*2)
            zAngle -= Math.PI*2;

            if (!isVRPresenting()) {
                xAngle += yDelta*0.0025;
                while (xAngle < -Math.PI*0.5)
                xAngle = -Math.PI*0.5;
                while (xAngle > Math.PI*0.5)
                xAngle = Math.PI*0.5;
            }
 */


        });
        m
    }
}

impl Actor for FirstPersonCamera {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add main camera to scene
        {
            let go = world.new_game_object();
            let cam = Camera::default();
            let c = go.borrow_mut().add_component(cam);

            self.camera = Some(c);
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        for evt in world.events().iter() {
            self.handle_event(evt)
        }

        let cam = world.current_camera().unwrap();

        let mut handlers = Vec::new();
        handlers.append(&mut self.handlers);

        for &(ref mv, _, ref h) in handlers.iter() {
            if self.state.contains(*mv) {
                h(self, world.delta_time());
            }
        }

        self.handlers.append(&mut handlers);

        self.update_camera();

        // Update Camera
        {
            cam.borrow_mut().lookat(
                &Point3::from_homogeneous(self.eye.extend(1.0)),
                &Point3::from_homogeneous((self.eye + self.eye_dir * 10.0).extend(1.0)),
                &Vector3::new(0.0, 1.0, 0.0),
            );
        }
    }
}

impl FirstPersonCamera {
    pub fn update_camera(&mut self) {
        let cam = self.camera();

        // Update Camera
        {
            cam.borrow_mut().lookat(
                &Point3::from_homogeneous(self.eye.extend(1.0)),
                &Point3::from_homogeneous((self.eye + self.eye_dir * 10.0).extend(1.0)),
                &Vector3::new(0.0, 1.0, 0.0),
            );
        }
    }

    fn add<F>(&mut self, mv: Movement, key: &str, f: F)
    where
        F: Fn(&mut FirstPersonCamera, f64) + 'static,
    {
        self.handlers.push((mv, key.to_string(), Box::new(f)));
    }

    fn key_down(&mut self, input: &str) {
        for &(ref mv, ref key, _) in self.handlers.iter() {
            if input == key {
                self.state.insert(*mv)
            }
        }
    }

    fn key_up(&mut self, input: &str) {
        for &(ref mv, ref key, _) in self.handlers.iter() {
            if input == key {
                self.state.remove(*mv)
            }
        }
    }

    fn mouse_down(&mut self, input: usize) {
        if input == 0 && !self.clicked  {
            self.clicked = true;
            self.click_pos = self.mouse_pos.clone();
        }
    }

    fn mouse_up(&mut self, input: usize) {
        if input == 0 && self.clicked  {
            self.clicked = false;
        }
    }

    fn mouse_pos(&mut self, input: &(i32,i32)) {
        self.mouse_pos = Vector2::new(input.0,input.1);

        if self.clicked {
            self.state.insert(Movement::MOUSE);
        }else {
            self.state.remove(Movement::MOUSE);
        }
    }

    fn handle_event(&mut self, evt: &AppEvent) {
        match evt {
            &AppEvent::KeyUp(ref key) => {
                self.key_up(key.code.as_str());
            }

            &AppEvent::KeyDown(ref key) => {
                self.key_down(key.code.as_str());
            }

            &AppEvent::MouseDown(ref button_event) => {
                self.mouse_down(button_event.button);
            }

            &AppEvent::MouseUp(ref button_event) => {
                self.mouse_up(button_event.button);
            }

            &AppEvent::MousePos(ref pos_tuple) => {
                self.mouse_pos(pos_tuple);
            }

            _ => {}
        }
    }

    pub fn camera(&self) -> &ComponentType<Camera> {
        self.camera.as_ref().unwrap().try_as::<Camera>().unwrap()
    }
}

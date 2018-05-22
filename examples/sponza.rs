extern crate unrust;
#[macro_use]
extern crate unrust_derive;

use unrust::actors::{FirstPersonCamera, ShadowPass, SkyBox};
use unrust::engine::{AssetError, AssetSystem, DirectionalLight, GameObject, Light, Material, Mesh,
                     ObjMaterial, PointLight, Prefab, RenderQueue, TextureWrap};
use unrust::math::*;
use unrust::world::events::*;
use unrust::world::{Actor, Handle, Processor, World, WorldBuilder};

// GUI
use unrust::imgui;

use std::rc::Rc;

#[derive(Actor)]
pub struct MainScene {
    dir_light: Handle<GameObject>,
    point_light: Handle<GameObject>,

    animate_light: bool,
    last_event: Option<AppEvent>,
}

#[derive(Component)]
pub struct MaterialFilter {
    force_no_normal_map: bool,
}

impl Actor for MaterialFilter {}

impl Processor for MaterialFilter {
    fn new() -> MaterialFilter {
        MaterialFilter {
            force_no_normal_map: false,
        }
    }

    fn apply_materials(&self, materials: &Vec<Rc<Material>>) {
        for m in materials.iter() {
            m.set("uNoNormalMap", self.force_no_normal_map);
        }
    }

    fn watch_material() -> bool
    where
        Self: Sized,
    {
        return true;
    }
}

impl MainScene {
    fn new() -> MainScene {
        MainScene {
            last_event: None,
            animate_light: true,
            dir_light: GameObject::empty(),
            point_light: GameObject::empty(),
        }
    }
}

impl Actor for MainScene {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add direction light to scene.
        {
            let go = world.new_game_object();
            let mut light = DirectionalLight::default();
            light.direction = Vector3f::new(-0.8, -1.0, 0.0).normalize();
            go.borrow_mut().add_component(light);

            self.dir_light = go;
        }

        // add point light which on player'eye
        {
            let go = world.new_game_object();
            let mut point = PointLight::default();

            point.constant = 0.9;
            point.linear = 0.00000;
            point.quadratic = 0.000002;

            go.borrow_mut().add_component(point);
            self.point_light = go;
        }

        // Add 2 points light on the scene
        let points = [
            Vector3f::new(-1202.0, 161.0, 400.0),
            Vector3f::new(1122.0, 161.0, -450.0),
        ];

        for pos in points.iter() {
            let go = world.new_game_object();
            let mut point = PointLight::default();

            point.constant = 0.8;
            point.quadratic = 0.00001;
            point.linear = 0.0;

            go.borrow_mut().add_component(point);
            go.borrow_mut().add_component(Cube::new());

            let mut gtran = go.borrow_mut().transform.global();
            gtran.disp = *pos;

            go.borrow_mut().transform.set_global(gtran);
            go.borrow_mut()
                .transform
                .set_local_scale(Vector3f::new(10.0, 10.0, 10.0));
        }

        // Added the obj display
        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(WaveObjActor::new());
        }

        let fpc_ref = world.find_component::<FirstPersonCamera>().unwrap();
        let mut fpc = fpc_ref.borrow_mut();

        fpc.camera().borrow_mut().znear = 1.0;
        fpc.camera().borrow_mut().zfar = 10000.0;

        fpc.position = Vector3::new(0.0, 200.06, -3.36);
        fpc.direction = Vector3::new(-3.0, 0.0, -1.0).normalize();
        fpc.speed = 200.0;

        {
            let shadow_pass = world.find_component::<ShadowPass>();
            if let Some(sp) = shadow_pass {
                sp.borrow_mut().disable_cascaded();
            }
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        // Handle Events
        let normap_map_enabled;

        {
            let mut reset = false;
            let mut toggle_normal_map = false;

            for evt in world.events().iter() {
                self.last_event = Some(evt.clone());

                match evt {
                    &AppEvent::KeyUp(ref key) => match key.code.as_str() {
                        "KeyU" => toggle_normal_map = true,
                        "Space" => self.animate_light = !self.animate_light,
                        "Escape" => reset = true,

                        _ => (),
                    },

                    _ => (),
                }
            }

            if reset {
                world.reset();
                // Because reset will remove all objects in the world,
                // included this Actor itself
                // so will need to add it back.
                let scene = world.new_game_object();
                scene.borrow_mut().add_component(MainScene::new());
                return;
            }

            if toggle_normal_map {
                let mf = world.find_component::<MaterialFilter>().unwrap();
                let b = mf.borrow().force_no_normal_map;
                mf.borrow_mut().force_no_normal_map = !b;
            }

            normap_map_enabled = {
                let mf = world.find_component::<MaterialFilter>().unwrap();
                let mf_borrow = mf.borrow();
                !mf_borrow.force_no_normal_map
            };
        }

        // Update Direction light
        if self.animate_light {
            let dir_light_bor = self.dir_light.borrow_mut();
            let (mut light, _) = dir_light_bor.find_component_mut::<Light>().unwrap();

            let dir = light.directional().unwrap().direction;
            let t = Quaternion::from_angle_y(Rad(0.01));
            let transformed = t * dir;

            light.directional_mut().unwrap().direction = transformed;
        }

        // Update Point Light
        {
            let cam = world.current_camera().unwrap();

            let point_light_bor = self.point_light.borrow_mut();
            let light_opt = point_light_bor.find_component_mut::<Light>();

            if let Some((mut light, _)) = light_opt {
                light.point_mut().unwrap().position = cam.borrow().eye();
            }
        }

        // GUI
        use imgui::Metric::*;
        use imgui::TextAlign::*;

        imgui::pivot((1.0, 1.0));
        imgui::text_align(Left);
        imgui::label(
            Native(1.0, 1.0) - Pixel(8.0, 8.0),
            "[WASD ZXEC] : control camera\n[Space] : Toggle light animation\n[U] : Toggle normal map\n[Esc] : reload all (include assets)",
        );

        let fpc_ref = world.find_component::<FirstPersonCamera>().unwrap();
        let fpc = fpc_ref.borrow_mut();

        imgui::pivot((1.0, 0.0));
        imgui::text_align(Right);
        imgui::label(
            Native(1.0, 0.0) + Pixel(-8.0, 8.0),
            &format!(
                "last event: {:?}\nnormal_map = {:?}\nlight animation={:?}\nposition={:?}\ndirection={:?}",
                self.last_event, normap_map_enabled, self.animate_light, fpc.position,fpc.direction
            ),
        );

        imgui::pivot((0.5, 1.0));
        imgui::text_align(Center);
        imgui::label(Native(0.5, 1.0) + Pixel(0.0, -8.0), "Sponza Demo");
    }
}

#[derive(Actor)]
pub struct WaveObjActor {}

impl WaveObjActor {
    fn new() -> WaveObjActor {
        WaveObjActor {}
    }
}

fn build_material(asys: &AssetSystem, obj_mat: ObjMaterial) -> Rc<Material> {
    let shader_program = match obj_mat.normal_map {
        Some(_) => asys.new_program("obj_nm"),
        None => asys.new_program("obj"),
    };

    let mut material = Material::new(shader_program);

    material.set(
        "uMaterial.ambient",
        obj_mat.ambient.unwrap_or(Vector3::new(0.2, 0.2, 0.2)),
    );

    let diffuse_tex = asys.new_texture(&obj_mat.diffuse_map.unwrap_or("default_white".to_owned()));
    diffuse_tex.wrap_u.set(TextureWrap::Repeat);
    diffuse_tex.wrap_v.set(TextureWrap::Repeat);
    material.set(
        "uMaterial.diffuse",
        obj_mat.diffuse.unwrap_or(Vector3::new(1.0, 1.0, 1.0)),
    );
    material.set("uMaterial.diffuse_tex", diffuse_tex);

    let specular_tex =
        asys.new_texture(&obj_mat.specular_map.unwrap_or("default_black".to_owned()));
    specular_tex.wrap_u.set(TextureWrap::Repeat);
    specular_tex.wrap_v.set(TextureWrap::Repeat);
    material.set(
        "uMaterial.specular",
        obj_mat.specular.unwrap_or(Vector3::new(1.0, 1.0, 1.0)),
    );
    material.set("uMaterial.specular_tex", specular_tex);

    material.set("uMaterial.shininess", obj_mat.shininess.unwrap_or(32.0));
    material.set("uMaterial.transparent", obj_mat.alpha.unwrap_or(1.0));

    obj_mat.normal_map.as_ref().map(|nm| {
        let n_tex = asys.new_texture(nm);
        n_tex.wrap_u.set(TextureWrap::Repeat);
        n_tex.wrap_v.set(TextureWrap::Repeat);

        material.set("uMaterial.normal_map", n_tex);
    });

    match obj_mat.alpha_mask {
        Some(ref f) => material.set("uMaterial.mask_tex", asys.new_texture(&f)),
        None => material.set("uMaterial.mask_tex", asys.new_texture("default_white")),
    }

    if obj_mat.alpha.unwrap_or(1.0) < 0.9999 || obj_mat.alpha_mask.is_some() {
        material.render_queue = RenderQueue::Transparent;
    }

    Rc::new(material)
}

impl Actor for WaveObjActor {
    fn start_rc(&mut self, go: Handle<GameObject>, world: &mut World) {
        let db = &mut world.asset_system();

        let prefab_handler = {
            let go = go.clone();
            move |r: Result<Prefab, AssetError>| match r {
                Ok(prefab) => for c in prefab.meshes {
                    go.borrow_mut().add_component(c.clone());
                },
                Err(err) => {
                    panic!(format!("Cannot load prefab, reason:{:?}", err));
                }
            }
        };

        db.new_prefab(
            "sponza/sponza.obj",
            Box::new(build_material),
            Box::new(prefab_handler),
        );
    }
}

#[derive(Actor)]
pub struct Cube {
    start_pos: Option<Vector3f>,
    t: f64,
}

impl Cube {
    fn new() -> Cube {
        Cube {
            start_pos: None,
            t: 0.0,
        }
    }
}

impl Actor for Cube {
    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let material = Material::new(db.new_program("sponza/lightbox"));

        let mut mesh = Mesh::new();
        mesh.add_surface(db.new_mesh_buffer("cube"), material);
        go.add_component(mesh);
    }

    fn update(&mut self, go: &mut GameObject, world: &mut World) {
        let start_pos = match self.start_pos {
            Some(p) => p,
            None => {
                let t = go.transform.local().disp;
                self.start_pos = Some(t);
                t
            }
        };

        self.t += world.delta_time();
        let offset = Vector3f::unit_y() * (self.t.sin() as f32) * 100.0;

        let mut ltran = go.transform.local();
        let q = Quaternion::from(Euler::new(Rad(0.01), Rad(0.01), Rad(0.01)));
        ltran.rot = ltran.rot * q;
        ltran.disp = start_pos + offset;

        go.transform.set_local(ltran);
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("sponza demo")
        .with_size((800, 600))
        .with_stats(true)
        .with_processor::<MaterialFilter>()
        .with_processor::<SkyBox>()
        .with_processor::<FirstPersonCamera>()
        .with_processor::<ShadowPass>()
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}

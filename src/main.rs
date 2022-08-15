use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PresentMode,
    input::mouse::{MouseWheel, MouseScrollUnit},
    render::mesh::{self, PrimitiveTopology},
    winit::WinitSettings
};
use std::{
    f32::consts::TAU,
    str::FromStr
};

struct Redraw;

#[derive(Clone)]
struct Polygon {
    n: usize,
    k: usize
}
impl ToString for Polygon {
    fn to_string(&self) -> String {
        if self.k == 1 {
            format!("{}", self.n)
        } else {
            format!("{}/{}", self.n, self.k)
        }
    }
}
impl FromStr for Polygon {
    type Err = usize;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sections = s.split('/').collect::<Vec<&str>>();
        if sections.len() == 1 {
            match sections[0].parse::<usize>() {
                Ok(n) => Ok(Polygon {
                    n,
                    k: 1
                }),
                Err(_) => Err(0)
            }
        } else if sections.len() == 2 {
            if let Ok(i) = sections[0].parse::<usize>() {
                let n = i;
                if let Ok(i) = sections[1].parse::<usize>() {
                    let k = i;
                    Ok(Polygon {
                        n,
                        k
                    })
                } else {
                    Err(s.find('/').unwrap() + 1)
                }
            } else {
                Err(0)
            }
        } else {
            let mut index = 0;
            let mut slashes = 0;
            for ch in s.chars() {
                if ch == '/' {
                    slashes += 1
                }
                if slashes == 2 {
                    break
                }
                index += 1;
            }
            Err(index)
        }
    }
}

#[derive(Clone)]
struct Data {
    material: Handle<ColorMaterial>,
    vertex: Mesh2dHandle,
    polygon: Polygon,
    scale: f32
}
impl FromWorld for Data {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        let material = materials.add(ColorMaterial::from(Color::rgb(1.0, 1.0, 1.0)));
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let vertex = meshes.add(shape::Circle::new(3.0).into()).into();
        Data {
            material,
            vertex,
            polygon: Polygon { n: 5, k: 2 },
            scale: 100.0
        }
    }
}

fn redraw(mut event: EventReader<Redraw>, data: Res<Data>, meshes: ResMut<Assets<Mesh>>, mut commands: Commands, shapes: Query<Entity, Or<(With<Vertex>, With<Line>)>>) {
    if event.iter().len() > 0 {
        for shape in shapes.iter() {
            commands.entity(shape).despawn();
        }
        create_shape(commands, meshes, data)
    }
}

fn create_line_mesh(a: Vec3, b: Vec3) -> Mesh {
    let vertices = vec![[a.x, a.y, a.z], [b.x, b.y, b.z]];
    let normal = (a-b).normalize();
    let normals = vec![[normal.x, normal.y, normal.z], [-normal.x, -normal.y, -normal.z]];
    let uvs = vec![[0.0, 0.0], [1.0, 1.0]];
    let indices = mesh::Indices::U16(vec![0, 1]);

    let mut mesh = Mesh::new(PrimitiveTopology::LineList);
    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

#[derive(Component)]
struct Vertex;
#[derive(Component)]
struct Line;
#[derive(Component)]
struct InputText;

fn create_shape(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, data: Res<Data>) {
    let polygon = &data.polygon;
    let angle = Vec2::from_angle(TAU/(polygon.n as f32));
    let line_angle = Vec2::from_angle(TAU*(polygon.k as f32)/(polygon.n as f32));
    let mut previous = Vec2::new(1.0, 0.0);
    for _ in 0..polygon.n {
        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh: data.vertex.clone(),
            material: data.material.clone(),
            transform: Transform::from_translation(previous.extend(0.0)*data.scale),
            ..default()
        }).insert(Vertex);
        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(create_line_mesh(previous.extend(0.0)*data.scale, line_angle.rotate(previous).extend(0.0)*data.scale)).into(),
            material: data.material.clone(),
            ..default()
        }).insert(Line);
        previous = angle.rotate(previous)
    }
}

fn scale(
        mut scroll_events: EventReader<MouseWheel>, 
        mut data: ResMut<Data>,
        mut redraw_ev: EventWriter<Redraw>
    ) 
{
    let mut scroll = 0.0;
    for e in scroll_events.iter() {
        scroll += match e.unit {
            MouseScrollUnit::Line => e.y,
            MouseScrollUnit::Pixel => e.y*0.01
        }
    }
    if scroll != 0.0 {
        data.scale *= 1.1f32.powf(scroll);
        redraw_ev.send(Redraw);
    }
}

fn setup_input(mut commands: Commands, assets_server: Res<AssetServer>, data: Res<Data>) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands.spawn_bundle(TextBundle::from_sections([
        TextSection::new(
            "\n",
            TextStyle {
                font: assets_server.load("consola.ttf"),
                font_size: 25.0,
                color: Color::WHITE
            }
        ),
        TextSection::new(
            '{',
            TextStyle {
                font: assets_server.load("consola.ttf"),
                font_size: 25.0,
                color: Color::WHITE
            }
        ),
        TextSection::new(
            data.polygon.to_string(),
            TextStyle {
                font: assets_server.load("consola.ttf"),
                font_size: 25.0,
                color: Color::WHITE
            }
        ),
        TextSection::new(
            '}',
            TextStyle {
                font: assets_server.load("consola.ttf"),
                font_size: 25.0,
                color: Color::WHITE
            }
        ),
    ]).with_text_alignment(TextAlignment::BOTTOM_LEFT).with_style(Style {
        align_self: AlignSelf::Center,
        position_type: PositionType::Absolute,
        position: UiRect {
            bottom: Val::Px(0.0),
            ..default()
        },
        ..default()
    })).insert(InputText);
}

fn keyboard_input(input: Res<Input<KeyCode>>, mut texts: Query<&mut Text>, mut data: ResMut<Data>, mut redraw_ev: EventWriter<Redraw>) {
    for mut text in &mut texts {
        let mut t = text.sections[2].value.clone();
        if input.just_pressed(KeyCode::Key0) {
            t.push('0')
        }
        if input.just_pressed(KeyCode::Key1) {
            t.push('1')
        }
        if input.just_pressed(KeyCode::Key2) {
            t.push('2')
        }
        if input.just_pressed(KeyCode::Key3) {
            t.push('3')
        }
        if input.just_pressed(KeyCode::Key4) {
            t.push('4')
        }
        if input.just_pressed(KeyCode::Key5) {
            t.push('5')
        }
        if input.just_pressed(KeyCode::Key6) {
            t.push('6')
        }
        if input.just_pressed(KeyCode::Key7) {
            t.push('7')
        }
        if input.just_pressed(KeyCode::Key8) {
            t.push('8')
        }
        if input.just_pressed(KeyCode::Key9) {
            t.push('9')
        }
        if input.just_pressed(KeyCode::Slash) {
            t.push('/')
        }
        if input.just_pressed(KeyCode::Back) {
            t.pop();
        }
        text.sections[2].value = t;
        match text.sections[2].value.parse::<Polygon>() {
            Ok(p) => {
                text.sections[0].value = "\n".to_owned();
                data.polygon = p;
                redraw_ev.send(Redraw)
            },
            Err(i) => {
                text.sections[0].value = " ".repeat(i+1) + "v\n";
            }
        }
    }
}

pub struct Shaper2D;
impl Plugin for Shaper2D {
    fn build(&self, app: &mut App) {
        app.init_resource::<Data>()
            .add_event::<Redraw>()
            .add_startup_system(setup_input)
            .add_startup_system(create_shape)
            .add_system(keyboard_input)
            .add_system(scale)
            .add_system(redraw);
    }
}

fn main() {
    App::new()
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(WindowDescriptor {
            title: "Shaper 2D".to_owned(),
            width: 500.0,
            height: 500.0,
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Shaper2D)
        .run();
}

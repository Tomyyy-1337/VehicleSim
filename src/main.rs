use nannou::{event::Update, glam::Vec2, App, Frame};
use rand::Rng;
use std::sync::mpsc::channel;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use nannou_egui::{self, egui, Egui};

mod vehicle;
use vehicle::Vehicle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildMode {
    Place,
    Remove,
    Inactive,
}


fn main() {
    nannou::app(Model::new).update(Model::update).run();
}

struct Model {
    pub egui: Egui, 
    width: f32,
    height: f32,
    vehicles: Vec<Vehicle>,
    lights: Vec<Vec2>,
    mouse_light: Vec2,
    light_intensity: f32,
    mouse_intensity: f32,
    car_intensity: f32,
    build_mode: BuildMode,
}

impl Model {
    fn new(app: &nannou::App) -> Self {
        let window_id = app.new_window()
            .size(800 as u32, 600 as u32)
            .view(Model::view)
            .raw_event(raw_window_event)
            .resized(Model::resized)
            .build()
            .unwrap();

        let window = app.window(window_id).unwrap();

        let mut rng = rand::thread_rng();
        let random_lights = (0..100).map(|_| Vec2::new(
            rng.gen_range(-800.0 / 2.0..800.0 / 2.0),
            rng.gen_range(-600.0 / 2.0..600.0 / 2.0),
        )).collect();

        Model {
            egui: Egui::from_window(&window),
            width: 800.0,
            height: 600.0,
            vehicles: vec![Vehicle::new()],
            lights: random_lights,
            mouse_light: Vec2::new(800.0 / 2.0, 600.0 / 2.0),
            light_intensity: 30.0,
            mouse_intensity: 1000.0,
            car_intensity: 10.0,
            build_mode: BuildMode::Inactive,
        }
    }

    fn resized(_app: &App, model: &mut Model, size: Vec2) {
        model.width = size.x;
        model.height = size.y;
        let mut rng = rand::thread_rng();
        let area = (model.width * model.height / 4000.0) as usize;
        model.lights = (0..area).map(|_| Vec2::new(
            rng.gen_range(-model.width / 2.0..model.width / 2.0),
            rng.gen_range(-model.height / 2.0..model.height / 2.0),
        )).collect();
    }

    fn update(app: &App, model: &mut Self, update: Update) {
        model.update_gui(update);

        if app.keys.down.contains(&nannou::event::Key::Space) {
            model.vehicles.push(Vehicle::new());
        } else if app.keys.down.contains(&nannou::event::Key::R) {
            model.vehicles.clear();
            model.vehicles.push(Vehicle::new());
        }

        match model.build_mode {
            BuildMode::Place if app.mouse.buttons.left().is_down() => {
                let mouse_position = app.mouse.position();
                let mut rng = rand::thread_rng();
                model.lights.push(Vec2::new(
                    rng.gen_range(mouse_position.x - 10.0..mouse_position.x + 10.0),
                    rng.gen_range(mouse_position.y - 10.0..mouse_position.y + 10.0),
                ));     
            },
            BuildMode::Remove if app.mouse.buttons.left().is_down() => {
                let mouse_position = app.mouse.position();
                model.lights
                    .retain(|light| (mouse_position - *light).length() > 50.0);
            },
            _ => {},
        }

        let delta = update.since_last.as_secs_f32();

        if app.mouse.position() != Vec2::ZERO {
            model.mouse_light = app.mouse.position();
        }

        let vehicles_coppy = model.vehicles.clone();
        
        for vehicle in &mut model.vehicles {
            vehicle.velocity = Vec2::ZERO;

            vehicle.adjust(&model.mouse_light, model.mouse_intensity);

            for light in model.lights.iter() {
                vehicle.adjust(light, model.light_intensity);
            }

            for other_vehicle in vehicles_coppy.iter() {
                if other_vehicle.position != vehicle.position {
                    vehicle.adjust(&other_vehicle.position, model.car_intensity);
                }
            }

            vehicle.velocity = vehicle.velocity.clamp_length_max(200.0);

            vehicle.update(delta);
        }

        model.vehicles.retain(|vehicle| {
            vehicle.position.x.abs() < model.width / 2.0 && vehicle.position.y.abs() < model.height / 2.0
        });
    }

    fn view(app: &App, model: &Self, frame: Frame) {
        let draw = app.draw();
        draw.background().color(nannou::color::BLACK);

        let tile_size = 16.0;

        // for i in 0..=(model.width as i32 / tile_size as i32) {
        //     for j in 0..=model.height as i32 / tile_size as i32 {
        //         let x_center = i as f32 * tile_size + 10.0 - model.width / 2.0;
        //         let y_center = j as f32 * tile_size + tile_size - model.height / 2.0;
        //         let mut light_intensity = model.lights.iter().fold(0.0, |acc, light| {
        //             acc + model.light_intensity * 3.0 / (Vec2::new(x_center, y_center) - *light).length().powi(2)
        //         });
        //         light_intensity += model.mouse_intensity * 3.0 / (Vec2::new(x_center, y_center) - model.mouse_light).length().powi(2);
        //         let rgb_color = nannou::color::rgb(light_intensity, 0.0, light_intensity);
        //         draw.rect()
        //             .x_y(x_center, y_center)
        //             .w_h(tile_size, tile_size)
        //             .color(rgb_color);
        //     }
        // }


        let (tx, rx) = channel();

        let light_intensity = model.light_intensity;
        let car_intensity = model.car_intensity;
        let mouse_intensity = model.mouse_intensity;
        let mouse_light = model.mouse_light;
        let height = model.height;
        let width = model.width;
        let lights = &model.lights;
        let vehicles = &model.vehicles;

        (0..=model.width as i32 / tile_size as i32).into_par_iter().for_each_with(tx, |tx, i| {
            let result = (-1..=height as i32 / tile_size as i32).into_iter().map(|j| {
                let x_center = i as f32 * tile_size + 10.0 - width / 2.0;
                let y_center = j as f32 * tile_size + tile_size - height / 2.0;
                // let mut light_intensity = model.lights.iter().fold(0.0, |acc, light| {
                //     acc + model.light_intensity * 3.0 / (Vec2::new(x_center, y_center) - *light).length().powi(2)
                // });
                // for vehicle in &model.vehicles {
                //     light_intensity += model.car_intensity * 3.0 / (Vec2::new(x_center, y_center) - vehicle.position).length().powi(2);
                // }

                let red = lights.iter().fold(0.0, |acc, light| {
                    acc + light_intensity * 3.0 / (Vec2::new(x_center, y_center) - *light).length().powi(2)
                });
                
                let mut blue = vehicles.iter().fold(0.0, |acc, vehicle| {
                    acc + car_intensity * 3.0 / (Vec2::new(x_center, y_center) - vehicle.position).length().powi(2)
                });
                blue += mouse_intensity * 3.0 / (Vec2::new(x_center, y_center) - mouse_light).length().powi(2);

                let green = 0.0;
                

                // light_intensity += model.mouse_intensity * 3.0 / (Vec2::new(x_center, y_center) - model.mouse_light).length().powi(2);
                // let rgb_color = nannou::color::rgb(light_intensity, 0.0, light_intensity);
                let rgb_color = nannou::color::rgb(red, green, blue);
                (x_center, y_center, rgb_color)
            }).collect::<Vec<_>>();
            tx.send(result).unwrap();
        });
    
        for (x_center, y_center, rgb_color) in rx.iter().flatten() {
            draw.rect()
                .x_y(x_center, y_center)
                .w_h(tile_size, tile_size)
                .color(rgb_color);
        }

        for vehicle in &model.vehicles {
            draw.rect()
                .x_y(vehicle.position.x, vehicle.position.y)
                .w_h(20.0, 10.0)
                .color(nannou::color::WHITE)
                .rotate(vehicle.velocity.y.atan2(vehicle.velocity.x));
        }

        for light in &model.lights {
            draw.ellipse()
                .x_y(light.x, light.y)
                .w_h(10.0, 10.0)
                .color(nannou::color::RED);
        }

        draw.ellipse()
            .x_y(model.mouse_light.x, model.mouse_light.y)
            .w_h(20.0, 20.0)
            .color(nannou::color::BLUE);

        draw.to_frame(app, &frame).unwrap();
        model.egui.draw_to_frame(&frame).unwrap();
    }

    pub fn update_gui(&mut self, update: Update) {
        self.egui.set_elapsed_time(update.since_start);
        let ctx = self.egui.begin_frame();
        egui::Window::new("Settings").show(&ctx, |ui| {
            ui.label("Mouse light intensity");
            ui.add(egui::Slider::new(&mut self.mouse_intensity, 0.0..=5000.0));
            ui.label("Car light intensity");
            ui.add(egui::Slider::new(&mut self.car_intensity, 0.0..=100.0));
            ui.label("Light intensity");
            ui.add(egui::Slider::new(&mut self.light_intensity, 0.0..=100.0));
            ui.label("Build mode");
            nannou_egui::egui::ComboBox::from_label("")
                .selected_text(match self.build_mode {
                    BuildMode::Place => "Place",
                    BuildMode::Remove => "Remove",
                    BuildMode::Inactive => "Inactive",
                })
                .show_ui(ui, |ui|{
                ui.selectable_value(&mut self.build_mode, BuildMode::Place, "Place");
                ui.selectable_value(&mut self.build_mode, BuildMode::Remove, "Remove");
                ui.selectable_value(&mut self.build_mode, BuildMode::Inactive, "Inactive");
            });
        });
    
    }

}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}
use ggegui::egui::Align2;
use ggegui::{egui, Gui};
use ggez::graphics::{
    Camera3d, Canvas3d, DrawParam3d, InstanceArray3d, Mesh3d, Mesh3dBuilder, Shader, ShaderBuilder,
};
use ggez::{glam, GameError};
use rand::Rng;
use std::sync::Arc;
use std::{env, path};

use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};

struct MainState {
    camera: Camera3d,
    instances: InstanceArray3d,
    instance_shader: Shader,
    custom_shader: Shader,
    gui: Gui,
    cube: Mesh3d,
    instance_shader_code: String,
    shader_code: String,
    instance_edit: bool,
    editing: bool,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut camera = Camera3d::default();
        camera.transform.yaw = 90.0;
        let cube = Mesh3dBuilder::new().cube(Vec3::splat(1.0)).build(ctx);
        let pyramid = Mesh3dBuilder::new()
            .pyramid(Vec2::splat(1.0), 2.0, false)
            .build(ctx);

        let mut instances = graphics::InstanceArray3d::new(ctx, None, pyramid);
        instances.resize(ctx, 100);

        let instance_shader_code = include_str!("../resources/instance_unordered3d.wgsl");
        let shader_code = include_str!("../resources/fancy.wgsl");

        Ok(MainState {
            camera,
            instances,
            instance_shader: ShaderBuilder::from_path("/instance_unordered3d.wgsl").build(ctx)?,
            custom_shader: ShaderBuilder::from_path("/fancy.wgsl").build(ctx)?,
            gui: Gui::new(ctx),
            cube,
            instance_shader_code: instance_shader_code.to_string(),
            shader_code: shader_code.to_string(),
            instance_edit: true,
            editing: false,
        })
    }
}

impl event::EventHandler for MainState {
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        _input: KeyInput,
        _repeated: bool,
    ) -> Result<(), GameError> {
        Ok(())
    }

    fn resize_event(&mut self, _: &mut Context, width: f32, height: f32) -> GameResult {
        self.camera.projection.resize(width as u32, height as u32);
        self.camera.projection.zfar = 10000.0;
        self.camera.projection.znear = 0.1;
        Ok(())
    }

    fn text_input_event(&mut self, ctx: &mut Context, character: char) -> GameResult {
        println!("{character}");
        self.gui.input.text_input_event(character, ctx);
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // GUI
        let gui_ctx = self.gui.ctx();

        egui::SidePanel::left("Code")
            .default_width(600.0)
            .show(&gui_ctx, |ui| {
                self.editing = CodeEditor::default()
                    .id_source("code editor")
                    .with_rows(12)
                    .with_fontsize(14.0)
                    .with_theme(ColorTheme::GRUVBOX)
                    .with_syntax(Syntax::rust())
                    .with_numlines(true)
                    .show(ui, {
                        if self.instance_edit {
                            &mut self.instance_shader_code
                        } else {
                            &mut self.shader_code
                        }
                    })
                    .has_focus();
            });

        egui::Window::new("UI")
            .anchor(Align2::RIGHT_TOP, [-25.0, 25.0])
            .show(&gui_ctx, |ui| {
                ui.toggle_value(&mut self.instance_edit, "Toggle Shader Type");
                if ui.button("compile").clicked() {
                    if self.instance_edit {
                        if let Ok(shader) =
                            ShaderBuilder::from_code(self.instance_shader_code.clone()).build(ctx)
                        {
                            self.instance_shader = shader;
                        } else {
                            println!("Failed");
                        }
                    } else {
                        if let Ok(shader) =
                            ShaderBuilder::from_code(self.shader_code.clone()).build(ctx)
                        {
                            self.custom_shader = shader;
                        } else {
                            println!("Failed");
                        }
                    }
                }
                if ui.button("quit").clicked() {
                    ctx.request_quit();
                }
            });
        self.gui.update(ctx);

        // Input
        if !self.editing {
            let k_ctx = &ctx.keyboard.clone();
            let (yaw_sin, yaw_cos) = self.camera.transform.yaw.sin_cos();
            let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
            let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

            if k_ctx.is_key_pressed(KeyCode::Space) {
                self.camera.transform.position.y += 1.0;
            }
            if k_ctx.is_key_pressed(KeyCode::C) {
                self.camera.transform.position.y -= 1.0;
            }
            if k_ctx.is_key_pressed(KeyCode::W) {
                self.camera.transform = self.camera.transform.translate(forward);
            }
            if k_ctx.is_key_pressed(KeyCode::S) {
                self.camera.transform = self.camera.transform.translate(-forward);
            }
            if k_ctx.is_key_pressed(KeyCode::D) {
                self.camera.transform = self.camera.transform.translate(right);
            }
            if k_ctx.is_key_pressed(KeyCode::A) {
                self.camera.transform = self.camera.transform.translate(-right);
            }
            if k_ctx.is_key_pressed(KeyCode::Right) {
                self.camera.transform.yaw += 1.0_f32.to_radians();
            }
            if k_ctx.is_key_pressed(KeyCode::Left) {
                self.camera.transform.yaw -= 1.0_f32.to_radians();
            }
            if k_ctx.is_key_pressed(KeyCode::Up) {
                self.camera.transform.pitch += 1.0_f32.to_radians();
            }
            if k_ctx.is_key_pressed(KeyCode::Down) {
                self.camera.transform.pitch -= 1.0_f32.to_radians();
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas3d = Canvas3d::from_frame(ctx, Color::new(0.25, 0.25, 0.25, 1.0));
        canvas3d.set_projection(self.camera.to_matrix());
        canvas3d.set_shader(&self.custom_shader);
        // Inverted box
        canvas3d.draw(&self.cube, DrawParam3d::default());
        canvas3d.set_shader(&self.instance_shader);

        // Set rotation, position, and color for boids
        self.instances.set((0..100).map(|i| {
            graphics::DrawParam3d::default()
                .position(Vec3::new(i as f32 * 2.0, 0.0, 0.0))
                .color(Color::WHITE)
        }));

        // Params that affect all boids
        let param = graphics::DrawParam3d::default()
            .color(Color::new(1.0, 1.0, 1.0, 1.0))
            .scale(Vec3::splat(2.0));

        canvas3d.draw(&self.instances, param);

        canvas3d.finish(ctx)?;
        let mut canvas = graphics::Canvas::from_frame(ctx, None);

        let dest_point1 = Vec2::new(10.0, 210.0);
        canvas.draw(
            &graphics::Text::new("You can mix 3d and 2d drawing;"),
            dest_point1,
        );
        canvas.draw(
            &self.gui,
            graphics::DrawParam::default().dest(glam::Vec2::ZERO),
        );
        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("3dshapes", "ggez")
        .window_mode(ggez::conf::WindowMode::default().resizable(true))
        .add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}

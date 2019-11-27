use sdl2::keyboard::Keycode;
use sdl2::event::{Event, WindowEvent};
use sprowl::{
    cgmath::{Matrix4, Vector2, Vector3, Vector4},
    smallvec::SmallVec,
    Error as SprowlError,
    color::Color,
    Canvas,
    gelem::{RenderStem, GraphicElement},
    font_renderer::FontStemDrawCall,
    shader::{BaseShader, Shader, ShaderDrawCall, CommonShaderDrawParams, RenderSource, self},
    utils::{Origin, DrawPos},
};

static FRAGMENT_SHADER_SOURCE: &'static str = include_str!("advanced_fs.glsl");
static VERTEX_SHADER_SOURCE: &'static str = include_str!("advanced_vs.glsl");

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ExampleUniformName {
    View,
    Model,
    OutlineColor,
    OutlineThickness,
    BackgroundColor,
    Effect,
    IsGrayscale,
    T,
}

impl shader::Uniform for ExampleUniformName {
    fn name(&self) -> &str {
        use ExampleUniformName::*;
        match self {
            View => "view",
            Model => "model",
            OutlineColor => "outline_color",
            OutlineThickness => "outline_thickness",
            Effect => "effect",
            BackgroundColor => "background_color",
            IsGrayscale => "is_grayscale",
            T => "t",
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub struct ExampleRenderParams {
    pub draw_pos: DrawPos,
    pub rotate: Option<(f32, Origin)>,
    pub scale: Option<f32>,
    pub outline: Option<Color<u8>>,
    pub effect: u32, // stub, glowing effect == 1 for now
    pub background_color: Option<Color<u8>>,
    pub t: f32,
}

impl ExampleRenderParams {
    pub fn new(pos: Vector2<i32>, origin: Origin) -> ExampleRenderParams {
        ExampleRenderParams {
            draw_pos: DrawPos { pos, origin },
            rotate: Default::default(),
            scale: Default::default(),
            outline: Default::default(),
            effect: 0,
            background_color: Default::default(),
            t: 0.0,
        }
    }
}

pub struct ExampleDrawCall {
    pub source: RenderSource,
    pub common: CommonShaderDrawParams,
    pub outline: Option<Color<u8>>,
    pub effect: u32,
    pub background_color: Option<Color<u8>>,
    pub t: f32,
}

pub struct ExampleShader {
    shader: BaseShader<ExampleUniformName>, 
    zoom_level: f32,
}

impl ExampleShader {
    pub fn new() -> Result<ExampleShader, shader::ShaderLoadError> {
        let basic_shader = BaseShader::new(FRAGMENT_SHADER_SOURCE, VERTEX_SHADER_SOURCE)?;
        let mut advanced_shader = ExampleShader { shader: basic_shader, zoom_level: 1.0 };
        advanced_shader.init_all_uniform_locations();
        Ok(advanced_shader)
    }
}

impl AsRef<BaseShader<ExampleUniformName>> for ExampleShader {
    fn as_ref(&self) -> &BaseShader<ExampleUniformName> {
        &self.shader
    }
}

impl AsMut<BaseShader<ExampleUniformName>> for ExampleShader {
    fn as_mut(&mut self) -> &mut BaseShader<ExampleUniformName> {
        &mut self.shader
    }
}

impl Shader for ExampleShader {
    type D = ExampleDrawCall;
    type U = ExampleUniformName;

    fn apply_draw_uniforms(&mut self, draw_call: Self::D) {
        use ExampleUniformName as UniName;

        let (width, height) = draw_call.render_source().size();
        let (scale_x, scale_y) = draw_call.common.scaling.compute_scale(width, height);
        let DrawPos {origin, pos} = draw_call.common.draw_pos;
        let (crop_offset_x, crop_offset_y, sprite_w, sprite_h) = draw_call.common.crop.unwrap_or((0, 0, width, height));
        let Vector2 { x: translate_origin_x, y: translate_origin_y } = origin.compute_relative_origin(Vector2::new(sprite_w, sprite_h));
        let mut model = Matrix4::from_nonuniform_scale((width as f32) * scale_x, (height as f32) * scale_y, 1.0);

        if let Some((angle, origin)) = draw_call.common.rotate {
            let Vector2 {x: pivot_x, y: pivot_y } = origin.compute_relative_origin(Vector2::new(sprite_w, sprite_h));
            let (pivot_x, pivot_y) = (pivot_x + crop_offset_x, pivot_y + crop_offset_y);
            model =
                // rotate around pivot center:
                // translate by (-width/2, -height/2)
                // then rotate,
                // then re-translate by (width/2, height/2)
                // YES this is the correct order, matrices multiplications should be read
                // from right to left!
                Matrix4::from_translation(Vector3::new(pivot_x as f32 * scale_x, pivot_y as f32 * scale_y, 0.0))
                * Matrix4::from_angle_z(cgmath::Deg(angle))
                * Matrix4::from_translation(Vector3::new(-pivot_x as f32 * scale_x, -pivot_y as f32 * scale_y, 0.0))
                * model
        }

        model = Matrix4::from_translation(Vector3::<f32>::new(
            pos.x as f32 - (translate_origin_x + crop_offset_x) as f32 * scale_x,
            pos.y as f32 - (translate_origin_y + crop_offset_y) as f32 * scale_y,
            0.0
        )) * model;

        self.shader.set_vector2(UniName::OutlineThickness, &Vector2::from((1.0 / width as f32 / scale_x, 1.0 / height as f32 / scale_y)));
        if let Some(outline_color) = draw_call.outline {
            let color = Vector4::from(outline_color.to_color_f32().rgba());
            self.shader.set_vector4(UniName::OutlineColor, &color);
        } else {
            self.shader.set_vector4(UniName::OutlineColor, &Vector4::from((0f32, 0f32, 0f32, 0f32)));
        }
        self.shader.set_uint(UniName::Effect, draw_call.effect);
        let bg_color = draw_call.background_color.unwrap_or(Color::from_rgba(0u8, 0, 0, 0));
        self.shader.set_vector4(UniName::BackgroundColor, &Vector4::from(bg_color.to_color_f32().rgba()));
        self.shader.set_float(UniName::T, draw_call.t);
        self.shader.set_uint(UniName::IsGrayscale, if draw_call.common.is_source_grayscale { 1 } else { 0 });
        self.shader.set_matrix4(UniName::Model, &model);
    }

    fn apply_global_uniforms(&mut self, (window_width, window_height): (u32, u32)) {
        let view_matrix = Matrix4::<f32>::from(cgmath::Ortho {
            left: 0.0,
            right: (window_width as f32) / self.zoom_level,
            bottom: (window_height as f32) / self.zoom_level,
            top: 0.0,
            near: -1.0,
            far: 1.0
        });
        self.shader.set_matrix4(ExampleUniformName::View, &view_matrix);
    }

    fn as_base_shader(&mut self) -> &mut BaseShader<Self::U> {
        &mut self.shader
    }

    fn init_all_uniform_locations(&mut self) {
        use ExampleUniformName::*;
        self.shader.init_uniform_location(Model);
        self.shader.init_uniform_location(View);
        self.shader.init_uniform_location(OutlineColor);
        self.shader.init_uniform_location(OutlineThickness);
        self.shader.init_uniform_location(Effect);
        self.shader.init_uniform_location(T);
        self.shader.init_uniform_location(IsGrayscale);
        self.shader.init_uniform_location(BackgroundColor);
    }
}

impl ShaderDrawCall for ExampleDrawCall {
    type RenderParams = ExampleRenderParams;

    fn render_source(&self) -> RenderSource {
        self.source
    }

    fn common_params(&self) -> &CommonShaderDrawParams {
        &self.common
    }

    fn from_graphic_elem<S: AsRef<str>>(
        graphic_elem: &GraphicElement<S, Self::RenderParams>,
        canvas: &mut Canvas
    ) -> Result<SmallVec<[Self; 2]>, SprowlError> {
        let mut results = SmallVec::new();

        let render_stem: &RenderStem<_> = &graphic_elem.render_stem;
        let render_params: &ExampleRenderParams = &graphic_elem.render_params;

        match render_stem {
            RenderStem::Texture { id: texture_id } => {
                let texture = canvas.get_texture(*texture_id).ok_or(SprowlError::MissingTextureId(*texture_id))?;
                let draw_call: ExampleDrawCall = ExampleDrawCall {
                    source: RenderSource::from(texture),
                    common: CommonShaderDrawParams::new(render_params.draw_pos),
                    outline: render_params.outline,
                    effect: render_params.effect,
                    background_color: render_params.background_color,
                    t: render_params.t,
                };
                results.push(draw_call);
            },
            RenderStem::Shape { shape } => {
                let draw_call: ExampleDrawCall = ExampleDrawCall {
                    source: RenderSource::from(shape),
                    common: CommonShaderDrawParams::new(render_params.draw_pos),
                    outline: render_params.outline,
                    effect: render_params.effect,
                    background_color: render_params.background_color,
                    t: render_params.t,
                };
                results.push(draw_call);
            },
            RenderStem::Text { font_id, font_size, text } => {
                let font_renderer = canvas.get_font_mut(*font_id).ok_or(SprowlError::MissingTextureId(*font_id))?;
                let characters: Vec<FontStemDrawCall> = font_renderer.word_to_draw_call(text.as_ref(), *font_size, render_params.draw_pos.pos);
                results.reserve(characters.len());
                for character in characters {
                    let mut common = CommonShaderDrawParams::new(DrawPos::new(character.dest_origin));
                    common.crop = Some(character.source_crop);
                    common.is_source_grayscale = true;
                    results.push(ExampleDrawCall {
                        source: RenderSource::from(character.texture),
                        common,
                        outline: render_params.outline,
                        effect: render_params.effect,
                        background_color: render_params.background_color,
                        t: render_params.t
                    })

                }
            }
        }

        Ok(results)
    }
}

fn run(sdl_context: &sdl2::Sdl, window: &sdl2::video::Window, mut canvas: Canvas) {
    let stick_id = canvas.add_texture_from_image_path("res/stick.png").unwrap();
    let font_id = canvas.add_font_from_bytes(include_bytes!("/usr/share/fonts/TTF/DejaVuSansMono.ttf"));
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut entity_x: i32 = 500;
    let mut entity_y: i32 = 500;

    static S_1: &str = "AZERTYUIOP^$QSDFGHJKLM%µWXCVBN";
    static S_2: &str = "Pote is SO kek";
    static S_3: &str = "O";
    static S_4: &str = "Well     spaced      text";

    let mut shader = ExampleShader::new().unwrap();

    'running: for t in 0.. {
        let text = match (t / 120) % 4 {
            0 => S_1,
            1 => S_2,
            2 => S_3,
            _ => S_4
        };

        let t0 = ::std::time::Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::Window { win_event: WindowEvent::SizeChanged(w, h), ..} => {
                    debug_assert!(w >= 0);
                    debug_assert!(h >= 0);
                    canvas.set_size((w as u32, h as u32));
                },
                Event::KeyDown { keycode: Some(Keycode::Up), repeat: false, ..} => {
                    entity_y -= 50;
                },
                Event::KeyDown { keycode: Some(Keycode::Down), repeat: false, ..} => {
                    entity_y += 50;
                },
                Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, ..} => {
                    entity_x -= 50;
                },
                Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, ..} => {
                    entity_x += 50;
                },
                _ => {}
            }
        }
        canvas.clear(Some(Color::from_rgb(128u8, 128, 128)));

        let mut graphic_elements: Vec<GraphicElement<&'static str, ExampleRenderParams>> = vec!();
        {
            let mut render_params = ExampleRenderParams::new(Vector2::new(entity_x, entity_y), Origin::Center);
            render_params.outline = Some(Color::from_rgb(255u8, 0, 255u8));
            graphic_elements.push(
                GraphicElement {
                    render_stem: RenderStem::Texture { id: stick_id },
                    render_params,
                }
            );
        }
        {
            let mut render_params = ExampleRenderParams::new(Vector2::new(0, 0), Origin::TopLeft(0, 0));
            render_params.outline = Some(Color::from_rgb(255u8, 0, 0));
            graphic_elements.push(
                GraphicElement {
                    render_stem: RenderStem::Text { font_id, text, font_size: 32.0 },
                    render_params,
                },
            );
        }
        canvas.draw(&mut shader, &graphic_elements);
        window.gl_swap_window();

        let _delta_t = ::std::time::Instant::now() - t0;
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 30));
    }
}

// fn advanced(sdl_context: &sdl2::Sdl, window: &sdl2::video::Window, mut canvas: Canvas) {
//     use sprowl::shaders::advanced::*;
//     let characters_id = canvas.add_texture_from_image_path("res/characters.png").unwrap();
//     let shapes_id = canvas.add_texture_from_image_path("res/shapes.png").unwrap();
//     let font_id = canvas.add_font_from_bytes(include_bytes!("/usr/share/fonts/TTF/DejaVuSansMono.ttf"));
//     let mut event_pump = sdl_context.event_pump().unwrap();


//     let mut shader = AdvancedShader::new().unwrap();

//     let mut outline = false;
//     let mut scale = false;

//     'running: for t in 0.. {
//         let t0 = ::std::time::Instant::now();
//         for event in event_pump.poll_iter() {
//             match event {
//                 Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
//                     break 'running
//                 },
//                 Event::Window { win_event: WindowEvent::SizeChanged(w, h), ..} => {
//                     debug_assert!(w >= 0);
//                     debug_assert!(h >= 0);
//                     canvas.set_size((w as u32, h as u32));
//                 },
//                 Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::A), repeat: false, .. } => {
//                     outline = !outline;
//                 },
//                 Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::E), repeat: false, .. } => {
//                     scale = !scale;
//                 },
//                 _ => {}
//             }
//         }
//         canvas.clear(Some(Color::from_rgb(128u8, 128, 128)));

//         let graphic_elements: Vec<GraphicElement<&'static str, AdvancedRenderParams>> = vec!(
//             GraphicElement {
//                 render_stem: RenderStem::Texture { id: shapes_id },
//                 render_params: RenderParams {
//                     common: CommonRenderParams::new(DrawPos { origin: Origin::Center , x: 300, y: 300 }),
//                     custom: AdvancedRenderParams {
//                         outline: if outline { Some(Color::from_rgb(0u8, 0, 255)) } else { None },
//                         rotate: None,
//                         scale: None,
//                         effect: 0,
//                         background_color: None,
//                         t: t as f32 / 10.0,
//                     }
//                 }
//             },
//             GraphicElement {
//                 render_stem: RenderStem::Texture { id: characters_id },
//                 render_params: RenderParams {
//                     common: CommonRenderParams {
//                         crop: Some((32, 160, 32, 32)),
//                         draw_pos: DrawPos { origin: Origin::new(), x: 100, y: 100 },
//                         is_source_grayscale: false,
//                     },
//                     custom: AdvancedRenderParams {
//                         outline: if outline { Some(Color::from_rgb(0u8, 0, 255)) } else { None },
//                         rotate: Some((t as f32, Origin::Center)),
//                         scale: if scale { Some(3.0) } else { None },
//                         effect: 0,
//                         background_color: None,
//                         t: t as f32 / 10.0,
//                     }
//                 }
//             },
//             GraphicElement {
//                 render_stem: RenderStem::Shape { shape: crate::Shape::Rect(200, 100) },
//                 render_params: RenderParams {
//                     common: CommonRenderParams::new(DrawPos { origin: Origin::Center, x: 200, y: 200 }),
//                     custom: AdvancedRenderParams {
//                         outline: None,
//                         rotate: None,
//                         scale: None,
//                         effect: 1,
//                         background_color: None,
//                         t: t as f32 / 10.0,
//                     }
//                 }
//             },
//             GraphicElement {
//                 render_stem: RenderStem::Shape { shape: crate::Shape::Rect(50, 50) },
//                 render_params: RenderParams {
//                     common: CommonRenderParams::new(DrawPos { origin: Origin::Center, x: 300, y: 300 }),
//                     custom: AdvancedRenderParams {
//                         outline: None,
//                         rotate: None,
//                         scale: None,
//                         effect: 2,
//                         background_color: Some(Color::from_rgba(64, 64, 64u8, 255u8)),
//                         t: t as f32 / 10.0,
//                     }
//                 }
//             },
//             GraphicElement {
//                 render_stem: RenderStem::Text { font_id, text: "Potekek", font_size: 30.0, max_width: None },
//                 render_params: RenderParams {
//                     common: CommonRenderParams::new(DrawPos { origin: Origin::Center, x: 0, y: 0 }),
//                     custom: AdvancedRenderParams {
//                         outline: if outline { Some(Color::from_rgb(0u8, 0, 255)) } else { None },
//                         rotate: Some((3.0 * t as f32, Origin::Center)),
//                         scale: if scale { Some(3.0) } else { None },
//                         effect: 0,
//                         background_color: Some(Color::from_rgb(128u8, 128u8, 128u8)),
//                         t: t as f32 / 10.0,
//                     }
//                 }
//             },
//         );
//         canvas.draw(&mut shader, &graphic_elements);
//         window.gl_swap_window();

//         let _delta_t = ::std::time::Instant::now() - t0;
//         // println!("{} fps (theory)", 1_000_000_000 / delta_t.subsec_nanos());
//         ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
//     }
// }

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(::sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    // // Enable anti-aliasing
    // gl_attr.set_multisample_buffers(1);
    // gl_attr.set_multisample_samples(1);
    
    let window = video_subsystem.window("Window", 1280, 720)
        .opengl()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    
    // Yes, we're still using the Core profile
    debug_assert_eq!(gl_attr.context_profile(), sdl2::video::GLProfile::Core);
    // ... and we're still using OpenGL 3.3
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    let canvas = {
        let (w, h) = window.size();
        Canvas::new((w, h))
    };

    run(&sdl_context, &window, canvas);
}
use sdl2::keyboard::Keycode;
use sdl2::event::{Event, WindowEvent};
use sprowl::{
    cgmath::{Matrix4, Vector2, Vector3, Vector4},
    smallvec::SmallVec,
    Error as SprowlError,
    Color,
    Canvas,
    font::{FontStemDrawCall, AdvancedLayoutIter, WordPos},
    render::{RenderStem, GraphicElement, RenderSource},
    shader::{BaseShader, Shader, Scaling, ShaderDrawCall, CommonShaderDrawParams, self},
    utils::{Shape, Origin, DrawPos},
};

use std::cmp::{max, min};

static FRAGMENT_SHADER_SOURCE: &'static str = include_str!("advanced_fs.glsl");
static VERTEX_SHADER_SOURCE: &'static str = include_str!("advanced_vs.glsl");

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ExampleUniformName {
    View,
    Model,
    Texture0,
    Texture1,
    OutlineColor,
    OutlineThickness,
    Effect,
    EffectColor,
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
            Texture0 => "texture0",
            Texture1 => "texture1",
            OutlineThickness => "outline_thickness",
            EffectColor => "effect_color",
            Effect => "effect",
            IsGrayscale => "is_grayscale",
            T => "t",
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Effect {
    None,
    Glowing(Color<u8>),
    Solid(Color<u8>),
    TextWave(Color<u8>),
}

#[derive(Debug, Copy, Clone)]
pub enum TextAlign {
    Left,
    Right,
    Center,
}

impl TextAlign {
    /// diff is the difference between the bounding box and the actual content bounding box.
    pub fn offset(&self, diff: u32) -> u32 {
        match self {
            TextAlign::Center => diff / 2,
            TextAlign::Right => diff,
            TextAlign::Left => 0
        }
    }
}

impl Default for TextAlign {
    fn default() -> TextAlign {
        TextAlign::Left
    }
}

#[derive(Debug, Copy, Clone)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

impl VerticalAlign {
    /// diff is the difference between the bounding box and the actual content bounding box.
    pub fn offset(&self, diff: u32) -> u32 {
        match self {
            VerticalAlign::Center => diff / 2,
            VerticalAlign::Top => 0,
            VerticalAlign::Bottom => diff,
        }
    }
}

impl Default for VerticalAlign {
    fn default() -> VerticalAlign {
        VerticalAlign::Center
    }
}

impl Default for Effect {
    fn default() -> Effect {
        Effect::None
    }
}

impl Effect {
    pub fn as_draw_params(&self) -> (u32, Color<u8>) {
        match *self {
            Effect::None => (0, Color::from_rgba(0,0,0,0)),
            Effect::Glowing(c) => (1, c),
            Effect::Solid(c) => (2, c),
            Effect::TextWave(c) => (3, c),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ExampleRenderParams {
    pub draw_pos: DrawPos,
    pub crop: Option<(i32, i32, u32, u32)>,
    pub rotate: Option<(f32, Origin)>,
    pub scale: Option<f32>,
    pub outline: Option<Color<u8>>,
    pub effect: Effect,
    /// id of the 2nd texture to bind
    pub second_bind: Option<u32>,
    /// bounding_box, text_align, vertical_align
    pub text_params: Option<(Vector2<u32>, TextAlign, VerticalAlign)>,
    pub t: f32,
}

impl ExampleRenderParams {
    pub fn new(pos: Vector2<i32>, origin: Origin) -> ExampleRenderParams {
        ExampleRenderParams {
            draw_pos: DrawPos { pos, origin },
            crop: Default::default(),
            rotate: Default::default(),
            scale: Default::default(),
            outline: Default::default(),
            effect: Default::default(),
            second_bind: Default::default(),
            text_params: Default::default(),
            t: 0.0,
        }
    }
}

pub struct ExampleDrawCall {
    pub source: RenderSource,
    pub common: CommonShaderDrawParams,
    pub outline: Option<Color<u8>>,
    pub effect: u32,
    pub effect_color: Color<u8>,
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

        let thickness_pixels = 1.0;
        self.shader.set_vector2(UniName::OutlineThickness, &Vector2::from((thickness_pixels / width as f32 / scale_x, thickness_pixels / height as f32 / scale_y)));
        if let Some(outline_color) = draw_call.outline {
            let color = Vector4::from(outline_color.to_color_f32().rgba());
            self.shader.set_vector4(UniName::OutlineColor, &color);
        } else {
            self.shader.set_vector4(UniName::OutlineColor, &Vector4::from((0f32, 0f32, 0f32, 0f32)));
        }
        self.shader.set_uint(UniName::Effect, draw_call.effect);
        self.shader.set_vector4(UniName::EffectColor, &Vector4::from(draw_call.effect_color.to_color_f32().rgba()));
        self.shader.set_float(UniName::T, draw_call.t);
        self.shader.set_uint(UniName::IsGrayscale, if draw_call.common.is_source_grayscale { 1 } else { 0 });
        self.shader.set_matrix4(UniName::Model, &model);
    }

    fn apply_global_uniforms(&mut self, (window_width, window_height): (u32, u32)) {
        self.shader.set_int(ExampleUniformName::Texture0, 0);
        self.shader.set_int(ExampleUniformName::Texture1, 1);

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
        self.shader.init_uniform_location(Texture0);
        self.shader.init_uniform_location(Texture1);
        self.shader.init_uniform_location(OutlineColor);
        self.shader.init_uniform_location(OutlineThickness);
        self.shader.init_uniform_location(Effect);
        self.shader.init_uniform_location(EffectColor);
        self.shader.init_uniform_location(T);
        self.shader.init_uniform_location(IsGrayscale);
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

        if let Some(tid) = render_params.second_bind {
            if let Some(t) = canvas.get_texture(tid) {
                t.bind(1);
            }
        }

        let (effect, effect_color) = render_params.effect.as_draw_params();

        match render_stem {
            RenderStem::Texture { id: texture_id } => {
                let texture = canvas.get_texture(*texture_id).ok_or(SprowlError::MissingTextureId(*texture_id))?;
                let mut common: CommonShaderDrawParams = CommonShaderDrawParams::new(render_params.draw_pos);
                common.crop = render_params.crop;
                common.rotate = render_params.rotate;
                common.scaling = render_params.scale.map(|s| Scaling::new(s)).unwrap_or(Scaling::None);
                let draw_call: ExampleDrawCall = ExampleDrawCall {
                    source: RenderSource::from(texture),
                    common,
                    outline: render_params.outline,
                    effect,
                    effect_color,
                    t: render_params.t,
                };
                results.push(draw_call);
            },
            RenderStem::Shape { shape } => {
                let mut common: CommonShaderDrawParams = CommonShaderDrawParams::new(render_params.draw_pos);
                common.crop = render_params.crop;
                common.rotate = render_params.rotate;
                common.scaling = render_params.scale.map(|s| Scaling::new(s)).unwrap_or(Scaling::None);
                let draw_call: ExampleDrawCall = ExampleDrawCall {
                    source: RenderSource::from(shape),
                    common,
                    outline: render_params.outline,
                    effect,
                    effect_color,
                    t: render_params.t,
                };
                results.push(draw_call);
            },
            RenderStem::Text { font_id, font_size, text } => {
                // necessary to avoid code duplication and make the borrow checker happy.
                let stem_to_real_draw_call = |character_stem_call: FontStemDrawCall<'_>| -> ExampleDrawCall {
                    let mut common = CommonShaderDrawParams::new(DrawPos::new(character_stem_call.dest_origin));
                    common.crop = Some(character_stem_call.source_crop);
                    common.is_source_grayscale = true;
                    common.pad = Some(1);
                    ExampleDrawCall {
                        source: RenderSource::from(character_stem_call.texture),
                        common,
                        outline: render_params.outline,
                        effect,
                        effect_color,
                        t: render_params.t
                    }
                };
                let font_renderer = canvas.get_font_mut(*font_id).ok_or(SprowlError::MissingTextureId(*font_id))?;
                if let Some(text_params) = render_params.text_params {
                    let topleft = render_params.draw_pos.pos - render_params.draw_pos.origin.compute_relative_origin(text_params.0);
                    let font_layout = AdvancedLayoutIter::new(font_renderer.font(), text.as_ref(), *font_size, Vector2::new(0.0, 0.0), text_params.0.x).collect::<Vec<_>>();
                    let actual_bb = font_layout
                        .iter()
                        .fold((0, 0), |(old_x, old_y), word| (
                            max(old_x as u32, word.origin.x as u32 + word.size.x as u32),
                            max(old_y as u32, word.origin.y as u32 + word.size.y as u32)
                        ));
                    let mut actual_bb = Vector2::new(actual_bb.0, actual_bb.1);
                    assert!(actual_bb.x <= text_params.0.x);
                    // this is to avoid an operator overflow, if the font_size is higher than the
                    // boudning box, we won't get an error.
                    actual_bb.y = min(actual_bb.y, text_params.0.y);
                    let diff = text_params.0 - actual_bb;
                    let offset = Vector2::new(text_params.1.offset(diff.x), text_params.2.offset(diff.y));
                    for WordPos { word, origin, .. } in font_layout {
                        let actual_pos = topleft + offset.cast::<i32>().unwrap() + origin.cast::<i32>().unwrap();
                        let characters = font_renderer.word_to_draw_call(word, *font_size, actual_pos);
                        results.reserve(characters.len());
                        for character in characters {
                            let draw_call = stem_to_real_draw_call(character);
                            results.push(draw_call);
                        };
                    };
                } else {
                    let characters = font_renderer.word_to_draw_call(text.as_ref(), *font_size, render_params.draw_pos.pos);
                    results.reserve(characters.len());
                    for character in characters {
                        let draw_call = stem_to_real_draw_call(character);
                        results.push(draw_call);
                    };
                };
            }
        }

        Ok(results)
    }
}

fn run(sdl_context: &sdl2::Sdl, window: &sdl2::video::Window, mut canvas: Canvas) {
    // add the resouces
    let stick_id = canvas.add_texture_from_image_path("res/stick.png").unwrap();
    let characters_id = canvas.add_texture_from_image_path("res/characters.png").unwrap();
    let shapes_id = canvas.add_texture_from_image_path("res/shapes.png").unwrap();
    let noise_id = canvas.add_texture_from_image_path("res/noise.png").unwrap();

    // font must always be from static resources, so use the include_bytes! macro.
    let font_id = canvas.add_font_from_bytes(include_bytes!("../res/DejaVuSerif.ttf"));
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut entity_x: i32 = 500;
    let mut entity_y: i32 = 500;

    static LOREM_IPSUM: &str = "AV.    Wa.     Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n\
Suspendisse congue bibendum odio, a vulputate diam condimentum vel. Quisque vestibulum tristique odio, ut faucibus mi gravida vel.";
    let mut shader = ExampleShader::new().unwrap();
    let mut scale: f32 = 1.0;

    log::info!("Running main loop...");
    'running: for t in 0.. {
        let loading_text = match (t / 20) % 4 {
            0 => "Loading",
            1 => "Loading.",
            2 => "Loading..",
            _ => "Loading...",
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
                Event::KeyDown { keycode: Some(Keycode::KpPlus), repeat: false, ..} => {
                    scale *= 2.0;
                },
                Event::KeyDown { keycode: Some(Keycode::KpMinus), repeat: false, ..} => {
                    scale *= 0.5;
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
        shader.zoom_level = scale;
        canvas.clear(Some(Color::from_rgb(192u8, 192, 192)));

        let mut graphic_elements: Vec<GraphicElement<&'static str, ExampleRenderParams>> = vec!();
        {
            // various shapes with outline
            let mut render_params = ExampleRenderParams::new(Vector2::new(0, 0), Origin::new());
            render_params.outline = Some(Color::from_rgb(255, 128, 0));
            graphic_elements.push( GraphicElement {
                render_stem: RenderStem::Texture { id: shapes_id },
                render_params,
            })
        }
        {
            // sprite with a border
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
            // regular text
            let mut render_params = ExampleRenderParams::new(Vector2::new(0, 0), Origin::TopLeft(0, 0));
            render_params.outline = Some(Color::from_rgb(0u8, 0u8, 0u8));
            graphic_elements.push(
                GraphicElement {
                    render_stem: RenderStem::Text { font_id, text: "Some Example with no BB & border: WAVE (kerning test)\n<- newline shows this", font_size: 32.0 },
                    render_params,
                },
            );
        }
        {
            // centered text
            let mut render_params = ExampleRenderParams::new(Vector2::new(0, 40), Origin::TopLeft(0, 0));
            render_params.outline = Some(Color::from_rgb(0u8, 0u8, 0u8));
            render_params.text_params = Some((Vector2::new(1280, 40), TextAlign::Center, Default::default()));
            graphic_elements.push(
                GraphicElement {
                    render_stem: RenderStem::Text { font_id, text: "Centered text (relative towindow)", font_size: 32.0 },
                    render_params,
                },
            );
        }
        {
            // middle text
            let mut render_params = ExampleRenderParams::new(Vector2::new(0, 0), Origin::TopLeft(0, 0));
            render_params.outline = Some(Color::from_rgb(0u8, 0u8, 0u8));
            render_params.text_params = Some((Vector2::new(1280, 720), TextAlign::Center, VerticalAlign::Center));
            graphic_elements.push(
                GraphicElement {
                    render_stem: RenderStem::Text { font_id, text: loading_text, font_size: 32.0 },
                    render_params,
                },
            );
        }
        {
            // multiline text
            let mut render_params = ExampleRenderParams::new(Vector2::new(0, 400), Origin::TopLeft(0, 0));
            render_params.outline = Some(Color::from_rgb(0u8, 0u8, 0u8));
            render_params.text_params = Some((Vector2::new(1280, 720), TextAlign::Left, VerticalAlign::Top));
            graphic_elements.push(
                GraphicElement {
                    render_stem: RenderStem::Text { font_id, text: LOREM_IPSUM, font_size: 48.0 },
                    render_params,
                },
            );
        }
        {
            // right-aligned text
            let mut render_params = ExampleRenderParams::new(Vector2::new(0, 80), Origin::TopLeft(0, 0));
            render_params.outline = Some(Color::black());
            // render_params.effect = Effect::TextWave(Color::from_rgb(43, 96, 222));
            render_params.effect = Effect::TextWave(Color::from_rgb(255, 255, 222));
            render_params.second_bind = Some(noise_id);
            render_params.t = t as f32;
            render_params.text_params = Some((Vector2::new(1280, 40), TextAlign::Right, Default::default()));
            graphic_elements.push(
                GraphicElement {
                    render_stem: RenderStem::Text { font_id, text: "Right-aligned text THING (relative to window)", font_size: 64.0 },
                    render_params,
                },
            );
        }
        {
            // solid shape
            let mut render_params = ExampleRenderParams::new(Vector2::new(300, 300), Origin::Center);
            render_params.effect = Effect::Solid(Color::from_rgb(64, 64, 64));
            graphic_elements.push( GraphicElement {
                render_stem: RenderStem::Shape { shape: Shape::Rect(50, 50) },
                render_params,
            })
        }
        {
            // glowing animated rect
            let mut render_params = ExampleRenderParams::new(Vector2::new(300, 300), Origin::Center);
            render_params.effect = Effect::Glowing(Color::from_rgb(64, 192, 1));
            render_params.t = t as f32 / 10.0;
            graphic_elements.push( GraphicElement {
                render_stem: RenderStem::Shape { shape: Shape::Rect(50, 50) },
                render_params,
            })
        }
        {
            // rotating sprite
            let mut render_params = ExampleRenderParams::new(Vector2::new(350, 350), Origin::Center);
            render_params.crop = Some((32, 32, 32, 32));
            render_params.rotate = Some((t as f32, Origin::Center));
            render_params.scale = Some(4.0);
            graphic_elements.push( GraphicElement {
                render_stem: RenderStem::Texture { id: characters_id },
                render_params,
            })
        }
        canvas.draw(&mut shader, &graphic_elements);
        window.gl_swap_window();

        let _delta_t = ::std::time::Instant::now() - t0;
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 30));
    }
}

fn main() {
    env_logger::init();
    log::info!("Starting program");
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

    log::info!("Initialized OpenGL, running...");

    // now that we are initialized, run the actual program
    run(&sdl_context, &window, canvas);
}
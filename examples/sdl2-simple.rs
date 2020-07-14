use sdl2::keyboard::Keycode;
use sdl2::event::{Event, WindowEvent};
use sprowl::{
    cgmath::{Matrix4, Vector2, Vector3, Vector4},
    Color,
    shader::{Shader, Uniform},
    renderer::{Renderer, RendererBuilder, AsVertexData},
    render_storage::{RenderStorage, texture::TextureArrayLayer, font::{AdvancedLayout, WordPos, FontStemDrawCall}, TextureKind, FontId},
};
use std::mem::transmute;
use std::cmp::min;

static FRAGMENT_SHADER_SOURCE: &'static str = include_str!("advanced_fs.glsl");
static VERTEX_SHADER_SOURCE: &'static str = include_str!("advanced_vs.glsl");

#[derive(Debug)]
pub enum GraphicElement {
    Rect(GraphicRect),
    Texture(GraphicTexture),
    Text(GraphicText),
}

static LOREM_IPSUM: &'static str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Fusce quis luctus leo, eget ultricies nisi. Phasellus gravida consequat viverra. Nam rhoncus euismod lectus id dictum. Sed finibus consequat orci a fermentum. Integer neque nulla, malesuada nec diam sed, eleifend tristique ante. Sed a hendrerit dui. Fusce tristique ante at feugiat venenatis. Phasellus molestie nulla vel arcu ultrices, quis bibendum metus lacinia. Etiam id iaculis purus. Etiam erat odio, pulvinar faucibus vulputate in, aliquam eu tellus. Duis placerat orci quis augue lacinia, dictum commodo magna fermentum.\n\n\
Maecenas a mollis quam. Ut vitae ligula ultricies, condimentum risus nec, vestibulum tellus. Aliquam lobortis velit in lorem molestie varius. Mauris in massa in nisl volutpat dignissim. Ut ex mi, pulvinar vel bibendum eu, porta ac metus. Suspendisse enim massa, tempus in facilisis sit amet, varius sollicitudin justo. Fusce tristique sollicitudin dui ac varius. Suspendisse sagittis lacus eu metus ultrices, vitae ornare urna lacinia. Fusce accumsan aliquam hendrerit. Nam fringilla metus condimentum, venenatis leo et, placerat ipsum. Maecenas enim arcu, facilisis et ultrices eget, ornare eget dui.\n\n\
Praesent condimentum enim quam, eget tincidunt massa rhoncus sed. Phasellus luctus aliquet magna, id pretium ipsum euismod eu. Nulla odio neque, porttitor ac felis eu, pellentesque sagittis eros. Cras scelerisque consequat ipsum, ut euismod diam rhoncus et. Aliquam erat volutpat. Praesent ornare vulputate nisi, et egestas quam aliquet sagittis. Morbi lorem libero, tincidunt eu semper ut, suscipit non urna. Morbi sodales elementum nunc at dapibus. Nullam nec lacus non urna tristique malesuada et eu tortor. Aenean tristique libero sed erat pellentesque, ac luctus nisi lacinia. Aliquam sit amet faucibus urna, eget efficitur tortor. Sed molestie tristique erat, quis condimentum felis.";

impl GraphicElement {
    pub fn draw_to_renderer(self, renderer: &mut Renderer<ExampleUniform>, render_storage: &mut RenderStorage) {
        match self {
            GraphicElement::Rect(r) => {
                renderer.add_elem(&VertexData {
                    position: Vector2::new(r.x as f32, r.y as f32),
                    size: Vector2::new(r.width as f32, r.height as f32),
                    rot_pivot: Vector2::new(r.width as f32 / 2.0, r.height as f32 / 2.0),
                    rot: r.rot,
                    crop: None,
                    kind: 2,
                    effect: 0,
                    layer: 0,
                    secondary_texture_layer: 0,
                    effect_color: r.color.to_color_f32().to_vec3(),
                })
            },
            GraphicElement::Texture(t) => {
                let stats = render_storage.get_stats(t.texture);
                let (width, height) = match t.crop {
                    Some((_, _, w, h,)) => (w, h),
                    None => (stats.width, stats.height),
                };
                let (max_w, max_h) = render_storage.get_max_dims(TextureKind::RGBA);
                let crop = match t.crop {
                    Some((x, y, w, h)) => {
                        (
                            x as f32 / max_w as f32,
                            y as f32 / max_h as f32,
                            w as f32 / max_w as f32,
                            h as f32 / max_h as f32,
                        )
                    },
                    None => {
                        (
                            0.0,
                            0.0,
                            width as f32 / max_w as f32,
                            height as f32 / max_h as f32,
                        )
                    }
                };
                renderer.add_elem(&VertexData {
                    position: Vector2::new(t.x as f32, t.y as f32),
                    size: Vector2::new(width as f32, height as f32),
                    rot_pivot: Vector2::new(width as f32 / 2.0, height as f32 / 2.0),
                    rot: t.rot,
                    crop: Some(crop),
                    kind: 0,
                    effect: 0,
                    layer: t.texture,
                    secondary_texture_layer: 0,
                    effect_color: Color::<f32>::black().to_vec3(),
                })
            },
            GraphicElement::Text(t) => {
                let (max_w, max_h) = render_storage.get_max_dims(TextureKind::Grayscale);
                let (font, mut texture) = render_storage.get_font_with_texture(t.font).unwrap();
                match t.width {
                    Some(max_width) => {
                        let font_layout = AdvancedLayout::new_str(
                            font.font(),
                            &t.text,
                            t.font_size,
                            Vector2::new(t.x as f32, t.y as f32),
                            t.center,
                            max_width
                        ).iter().cloned().collect::<Vec<WordPos<'_>>>();
                        for WordPos { word, origin, .. } in font_layout {
                            let word_layout = font.word_to_draw_call(
                                &mut texture, word, t.font_size
                            );
                            render_word(renderer, &word_layout, origin, (max_w, max_h));
                        };
                    },
                    None => {
                        let word_layout = font.word_to_draw_call(
                            &mut texture, &t.text, t.font_size
                        );
                        render_word(renderer, &word_layout, Vector2::new(t.x, t.y), (max_w, max_h));
                    }
                };
            },
        }
    }
}

pub fn render_word(renderer: &mut Renderer<ExampleUniform>, word_layout: &[FontStemDrawCall], origin: Vector2<f32>, texture_layer_dims: (u32, u32)) {
    let (max_w, max_h) = texture_layer_dims;
    for character in word_layout {
        let (w, h) = (character.source_crop.2, character.source_crop.3);
        let crop = Some((
            // 1 represents the padding for borders: we need the characters to be 1 pixel wider
            // to be able to show an outline.
            (character.source_crop.0 - 1f32) as f32 / max_w as f32,
            (character.source_crop.1 - 1f32) as f32 / max_h as f32,
            (character.source_crop.2 + 2f32) as f32 / max_w as f32,
            (character.source_crop.3 + 2f32) as f32 / max_h as f32,
        ));
        renderer.add_elem(&VertexData {
            position: Vector2::new((origin.x + character.dest_origin.x - 1f32) as f32, (origin.y + character.dest_origin.y - 1f32) as f32),
            size: Vector2::new((w + 2f32) as f32, (h + 2f32) as f32),
            rot_pivot: Vector2::new((w + 2f32) as f32 / 2.0, (h + 2f32) as f32 / 2.0),
            rot: 0.0,
            crop,
            kind: 1,
            effect: 8,
            layer: 0,
            secondary_texture_layer: 3, // "noise_id" layer in theory, but you shouldnt hardcode it...
            effect_color: Color::white().to_vec3(),
        });
    }
}

#[derive(Debug)]
pub struct GraphicRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rot: f32,
    pub color: Color<u8>,
}

#[derive(Debug)]
pub struct GraphicTexture {
    pub crop: Option<(i32, i32, u32, u32)>,
    pub x: i32,
    pub y: i32,
    pub rot: f32,
    pub texture: TextureArrayLayer,
    pub scale: Option<(f32, f32)>,
}

#[derive(Debug)]
pub struct GraphicText {
    pub x: f32,
    pub y: f32,
    pub width: Option<u32>,
    pub text: String,
    pub font_size: f32,
    pub font: FontId,
    pub center: i8,
}

#[derive(Debug, Copy, Clone)]
pub struct VertexData {
    crop: Option<(f32, f32, f32, f32)>,
    position: Vector2<f32>,
    size: Vector2<f32>,
    rot_pivot: Vector2<f32>,
    rot: f32,
    // first 8 bits ( ^ 0b1111111 ) => 0 = texture, 1 = text, 2 = rect
    //
    // then there are flags available for all other bits.
    kind: u32,
    layer: u32,
    secondary_texture_layer: u32,
    effect: u32,
    effect_color: Vector3<f32>,
}

impl AsVertexData for VertexData {
    fn add_vertex_data(&self, instanced_vb: &mut Vec<u8>) -> u32 {
        let crop = self.crop.map(|(x, y, w, h)| {
            Vector4::new(x, y, w, h)
        }).unwrap_or(Vector4::new(0.0, 0.0, 1.0, 1.0));
        unsafe {
            let b_crop = &transmute::<Vector4<f32>, [u8; 16]>(crop);
            instanced_vb.extend_from_slice(b_crop);

            let b_position = &transmute::<Vector2<f32>, [u8; 8]>(self.position);
            instanced_vb.extend_from_slice(b_position);

            let b_size = &transmute::<Vector2<f32>, [u8; 8]>(self.size);
            instanced_vb.extend_from_slice(b_size);

            let b_rot_pivot = &transmute::<Vector2<f32>, [u8; 8]>(self.rot_pivot);
            instanced_vb.extend_from_slice(b_rot_pivot);

            let b_rot = &transmute::<f32, [u8; 4]>(self.rot);
            instanced_vb.extend_from_slice(b_rot);

            let b_others = &transmute::<[u32; 4], [u8; 16]>(
                [self.kind, self.layer, self.secondary_texture_layer, self.effect]
            );
            instanced_vb.extend_from_slice(b_others);

            let b_effect_color = &transmute::<Vector3<f32>, [u8; 12]>(
                self.effect_color
            );
            instanced_vb.extend_from_slice(b_effect_color)
        }

        1
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ExampleUniform {
    View,
    T
}

impl Uniform for ExampleUniform {
    fn name(&self) -> &str {
        match self {
            ExampleUniform::View => "view",
            ExampleUniform::T => "t",
        }
    }

    fn for_each<F: FnMut(Self)>(mut f: F) {
        f(ExampleUniform::View);
        f(ExampleUniform::T);
    }
}

fn run(sdl_context: &sdl2::Sdl, window: &sdl2::video::Window) {
    let mut frames = 0u32;
    let mut compute_us = 0;
    let mut draw_us = 0;
    let mut swap_us = 0;
    let mut event_pump = sdl_context.event_pump().unwrap();

    if let Some(x) = sprowl::gl_utils::gl_get_error() {
        panic!("gl error code after initializing: {}", x);
    }

    let lorem_ipsum_length = LOREM_IPSUM.chars().count();

    let shader = Shader::<ExampleUniform>::new(
        FRAGMENT_SHADER_SOURCE,
        VERTEX_SHADER_SOURCE,
        &["texture_rgba", "texture_gray"]
    ).expect("error when creating shader");
    let mut renderer = RendererBuilder::new(16384)
        // layout = 1 -> vec4 crop 
        .with_instanced_vertex_attrib(4, gl::FLOAT)
        // vec2 translation
        .with_instanced_vertex_attrib(2, gl::FLOAT)
        // vec2 scale
        .with_instanced_vertex_attrib(2, gl::FLOAT)
        // vec2 rot_pivot
        .with_instanced_vertex_attrib(2, gl::FLOAT)
        // rot
        .with_instanced_vertex_attrib(1, gl::FLOAT)
        .with_instanced_vertex_attrib(1, gl::UNSIGNED_INT)
        .with_instanced_vertex_attrib(1, gl::UNSIGNED_INT)
        .with_instanced_vertex_attrib(1, gl::UNSIGNED_INT)
        .with_instanced_vertex_attrib(1, gl::UNSIGNED_INT)
        .with_instanced_vertex_attrib(3, gl::FLOAT)
        .build_with(shader);

    let mut render_storage = RenderStorage::new();

    // add the resouces
    let stick_id = render_storage.add_texture_from_image_bytes(include_bytes!("../res/stick.png"), None).unwrap();
    let characters_id = render_storage.add_texture_from_image_bytes(include_bytes!("../res/characters.png"), None).unwrap();
    let shapes_id = render_storage.add_texture_from_image_bytes(include_bytes!("../res/shapes.png"), None).unwrap();
    let _noise_id = render_storage.add_texture_from_image_bytes(include_bytes!("../res/noise.png"), None).unwrap();

    // font must always be from static resources, so use the include_bytes! macro.
    let font_id = render_storage.add_font_from_bytes(include_bytes!("../res/DejaVuSerif.ttf"));

    let mut current_size = window.drawable_size();

    log::info!("Running main loop...");
    let mut last_time = std::time::Instant::now();
    'running: for t in 0.. {
        log::info!("OpenGL Multisampling:                {}", unsafe { gl::IsEnabled(gl::MULTISAMPLE) });
        if let Some(e) = sprowl::gl_utils::gl_get_error() {
            panic!("opengl fatal error {}", e);
        }
        let t0 = ::std::time::Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::Window { win_event: WindowEvent::SizeChanged(w, h), ..} => {
                    debug_assert!(w >= 0);
                    debug_assert!(h >= 0);
                    renderer.set_viewport(w as u32, h as u32);
                    current_size = (w as u32, h as u32);
                },
                _ => {}
            }
        }

        renderer.clear(Some(Color::from_rgb(192u8, 192, 192)));
        let view_matrix = Matrix4::<f32>::from(cgmath::Ortho {
            left: 0.0,
            right: (current_size.0 as f32),
            bottom: current_size.1 as f32,
            top: 0.0,
            near: -1.0,
            far: 1.0
        });
        renderer.shader.set_matrix4(ExampleUniform::View, &view_matrix);
        renderer.shader.set_float(ExampleUniform::T, t as f32);

        for x in 0..64i32 {
            for y in 0..64i32 {
                let color = Color::from_rgb(255, (x * 2) as u8, (y * 5) as u8);
                let g = GraphicElement::Rect(GraphicRect { color, x: x * 25, y: y * 25, width: 20, height: 20, rot: t as f32 * 3.0});
                g.draw_to_renderer(&mut renderer, &mut render_storage);
            }
        }

        let text_progress = min(LOREM_IPSUM.len(), t / 4);
        // let text = if text_progress >= lorem_ipsum_length {
        //     String::from(LOREM_IPSUM)
        // } else {
        //     LOREM_IPSUM.chars().take(text_progress).collect::<String>()
        // };
        let text = format!("prout");
        let shapes = GraphicElement::Texture(GraphicTexture { texture: shapes_id, x: 0, y: 0, rot: 0.0, crop: None, scale: None});
        shapes.draw_to_renderer(&mut renderer, &mut render_storage);

        let stick = GraphicElement::Texture(GraphicTexture { texture: stick_id, x: 400, y: 400, rot: 0.0, crop: None, scale: None});
        stick.draw_to_renderer(&mut renderer, &mut render_storage);

        let sprite = GraphicElement::Texture(GraphicTexture { texture: characters_id, x: 0, y: 400, rot: t as f32 / 3.0, crop: Some((32, 32, 32, 32)), scale: Some((4.0, 4.0))});
        sprite.draw_to_renderer(&mut renderer, &mut render_storage);

        let text1 = GraphicElement::Text(GraphicText { x: 0.0, y: 0.0, font: font_id, width: Some(current_size.0), text, font_size: 50.0, center: -1});
        text1.draw_to_renderer(&mut renderer, &mut render_storage);

        let text2 = GraphicElement::Text(GraphicText {
            x: current_size.0 as f32 / 4.0, y: 50.0, font: font_id, width: Some(current_size.0 / 2),
            text:format!("Salut tout le monde comment ça va aujourd'hui"), font_size: 60.0, center: 0
        });
        text2.draw_to_renderer(&mut renderer, &mut render_storage);

        let text3 = GraphicElement::Text(GraphicText {
            x: current_size.0 as f32 / 4.0, y: 350.0, font: font_id, width: Some(current_size.0 / 2),
            text:format!("WAWAWA\nSALUT LES POTES\nWAW WAW\n\nXOXOXO WAW WAW WAW XOXOXO"), font_size: 60.0, center: 1
        });
        text3.draw_to_renderer(&mut renderer, &mut render_storage);

        render_storage.set_active();
        let t1 = std::time::Instant::now();
        renderer.draw();
        let t2 = std::time::Instant::now();

        window.gl_swap_window();
        let t3 = std::time::Instant::now();
        compute_us += (t1 - t0).as_micros();
        draw_us += (t2 - t1).as_micros();
        swap_us += (t3 - t2).as_micros();

        frames += 1;

        if (::std::time::Instant::now() - last_time).as_millis() >= 5000 {
            log::info!("current_fps: {:03}fps, compute={:05}us, draw={:05}us, swap={:05}us", frames / 5, compute_us / frames as u128, draw_us / frames as u128, swap_us / frames as u128);
            frames = 0;
            compute_us = 0;
            draw_us = 0;
            swap_us = 0;
            last_time = std::time::Instant::now();
        }
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
    
    let window = video_subsystem.window("Window", 1280, 720)
        .resizable()
        .opengl()
        .build()
        .unwrap();


    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    println!("MS {} {}", gl_attr.multisample_buffers(), gl_attr.multisample_samples());
    // Enable anti-aliasing
    gl_attr.set_multisample_buffers(0);
    gl_attr.set_multisample_samples(0);

    // unsafe { gl::Disable(gl::MULTISAMPLE); };

    println!("MS {} {}", gl_attr.multisample_buffers(), gl_attr.multisample_samples());
    
    // Yes, we're still using the Core profile
    debug_assert_eq!(gl_attr.context_profile(), sdl2::video::GLProfile::Core);
    // ... and we're still using OpenGL 3.3
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    if let Some(e) = sprowl::gl_utils::gl_get_error() {
        panic!("opengl fatal error {:x} while initializing", e);
    }


    video_subsystem.gl_set_swap_interval(sdl2::video::SwapInterval::Immediate).expect("failed to disable vsync");
    // now that we are initialized, run the actual program

    log::info!(
        "OpenGL Vendor: {}",
        sprowl::gl_utils::gl_get_string(gl::VENDOR).to_string_lossy(),
    );
    log::info!(
        "OpenGL Renderer: {}",
        sprowl::gl_utils::gl_get_string(gl::RENDERER).to_string_lossy(),
    );
    log::info!(
        "OpenGL Version: {}, GLSL Version: {}",
        sprowl::gl_utils::gl_get_string(gl::VERSION).to_string_lossy(),
        sprowl::gl_utils::gl_get_string(gl::SHADING_LANGUAGE_VERSION).to_string_lossy(),
    );
    log::info!("OpenGL MAX_TEXTURE_SIZE:             {}", sprowl::gl_utils::gl_get_int(gl::MAX_TEXTURE_SIZE));
    log::info!("OpenGL MAX_3D_TEXTURE_SIZE:          {}", sprowl::gl_utils::gl_get_int(gl::MAX_3D_TEXTURE_SIZE));
    log::info!("OpenGL MAX_ARRAY_TEXTURE_LAYERS:     {}", sprowl::gl_utils::gl_get_int(gl::MAX_ARRAY_TEXTURE_LAYERS));
    log::info!("OpenGL MAX_ELEMENTS_VERTICES:        {}", sprowl::gl_utils::gl_get_int(gl::MAX_ELEMENTS_VERTICES));
    log::info!("OpenGL MAX_ELEMENTS_INDICES:         {}", sprowl::gl_utils::gl_get_int(gl::MAX_ELEMENTS_INDICES));
    log::info!("OpenGL MAX_VERTEX_ATTRIBS:           {}", sprowl::gl_utils::gl_get_int(gl::MAX_VERTEX_ATTRIBS));
    log::info!("OpenGL MAX_UNIFORM_COMPONENTS:       {}", sprowl::gl_utils::gl_get_int(gl::MAX_VERTEX_UNIFORM_COMPONENTS));
    log::info!("OpenGL MAX_VERTEX_OUTPUT_COMPONENTS: {}", sprowl::gl_utils::gl_get_int(gl::MAX_VERTEX_OUTPUT_COMPONENTS));
    log::info!("OpenGL Multisampling:                {}", unsafe { gl::IsEnabled(gl::MULTISAMPLE) });
    // log::info!("OpenGL Multisampling ARB:                {}", unsafe { gl::IsEnabled(gl::MULTISAMPLE_ARB) });
    println!("MS {} {}", gl_attr.multisample_buffers(), gl_attr.multisample_samples());

    log::info!("Initialized OpenGL, running...");
    run(&sdl_context, &window);
}
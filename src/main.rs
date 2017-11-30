extern crate cgmath;
extern crate gl;
extern crate glfw;

use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, vec3};
use gl::types::*;
use glfw::{Action, Context, Key};
use std::collections::HashSet;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::str;

const TITLE: &str = "comanche";

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const VERTS: [f32; 24] = [
    -0.5, -0.5,  0.5,
     0.5, -0.5,  0.5,
     0.5,  0.5,  0.5,
    -0.5,  0.5,  0.5,
    -0.5, -0.5, -0.5,
     0.5, -0.5, -0.5,
     0.5,  0.5, -0.5,
    -0.5,  0.5, -0.5,
];
const INDICES: [i32; 36] = [
    0, 1, 2, 2, 3, 0, // +z
    5, 6, 7, 7, 4, 5, // -z
    3, 2, 6, 6, 7, 3, // +y
    4, 5, 1, 1, 0, 4, // -y
    1, 5, 6, 6, 2, 1, // +x
    4, 0, 3, 3, 7, 4, // -x
];

struct ShaderProgram {
    program: GLuint,
    mvp: GLint,
}

struct Camera {
    position: Vector3<f32>,
    direction: Vector3<f32>,
    keys: HashSet<Key>
}

unsafe fn check_status(obj: GLuint, param: GLenum) {
    let mut ok = gl::FALSE as GLint;
    let get_info = match param {
        gl::LINK_STATUS => {
            gl::GetProgramiv(obj, param, &mut ok);
            gl::GetProgramInfoLog
        },
        _ => {
            gl::GetShaderiv(obj, gl::COMPILE_STATUS, &mut ok);
            gl::GetShaderInfoLog
        },
    };
    if ok != gl::TRUE as GLint {
        let mut info = Vec::with_capacity(512);
        info.set_len(511);
        get_info(obj, 512, ptr::null_mut(), info.as_mut_ptr() as *mut GLchar);
        println!("ERROR::{}\n{}", param, str::from_utf8(&info).unwrap());
    }
}

unsafe fn compile_shader(kind: GLenum, src: &str) -> GLuint {
    let shader = gl::CreateShader(kind);
    let c_str = CString::new(src.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
    gl::CompileShader(shader);
    check_status(shader, kind);
    shader
}

fn setup_gl(win: &mut glfw::Window) -> (GLuint, ShaderProgram) {
    gl::load_with(|sym| win.get_proc_address(sym) as *const _);
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Viewport(0, 0, WIDTH as i32, HEIGHT as i32);
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
        gl::LogicOp(gl::INVERT);

        let vao = {
            let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (VERTS.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                           &VERTS[0] as *const f32 as *const c_void,
                           gl::STATIC_DRAW);
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (INDICES.len() * mem::size_of::<GLint>()) as GLsizeiptr,
                           &INDICES[0] as *const i32 as *const c_void,
                           gl::STATIC_DRAW);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * mem::size_of::<GLfloat>() as GLsizei, ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::BindVertexArray(0);
            vao
        };
        let program = {
            let p = gl::CreateProgram();
            let vs = compile_shader(gl::VERTEX_SHADER, include_str!("block_vert.glsl"));
            let fs = compile_shader(gl::FRAGMENT_SHADER, include_str!("block_frag.glsl"));
            gl::AttachShader(p, vs);
            gl::AttachShader(p, fs);
            gl::LinkProgram(p);
            check_status(p, gl::LINK_STATUS);
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
            ShaderProgram{
                program: p,
                mvp: gl::GetUniformLocation(p, CString::new("mvp").unwrap().as_ptr()),
            }
        };
        (vao, program)
    }
}

fn process_events(
    events: &std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
    win: &mut glfw::Window,
    camera: &mut Camera,
) {
    for (_, evt) in glfw::flush_messages(&events) {
        match evt {
            glfw::WindowEvent::FramebufferSize(w, h) => unsafe { gl::Viewport(0, 0, w, h) },
            glfw::WindowEvent::Key(key, _, action, _) => match (key, action) {
                (Key::Escape, Action::Press) => win.set_should_close(true),
                (key, Action::Press) => { camera.keys.insert(key); },
                (mut key, Action::Release) => { camera.keys.remove(&mut key); },
                _ => (),
            },
            _ => (),
        }
    }

    const SPEED: f32 = 0.5;
    let pos = camera.direction * SPEED;
    let up = vec3(0.0, 1.0, 0.0);
    if let Some(_) = camera.keys.get(&Key::W) { camera.position += pos }
    if let Some(_) = camera.keys.get(&Key::A) { camera.position -= pos.cross(up).normalize() }
    if let Some(_) = camera.keys.get(&Key::S) { camera.position -= pos }
    if let Some(_) = camera.keys.get(&Key::D) { camera.position += pos.cross(up).normalize() }
}

fn render(vao: GLuint, program: &ShaderProgram, camera: &mut Camera) {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        gl::UseProgram(program.program);
        let projection = cgmath::perspective(cgmath::Deg(45.0), 4.0 / 3.0, 0.1, 100.0);
        let view = Matrix4::look_at(
            Point3::from_vec(camera.position),
            Point3::from_vec(camera.position + camera.direction),
            vec3(0.0, 1.0, 0.0));
        let model = Matrix4::identity();
        let mvp = projection * view * model;
        gl::UniformMatrix4fv(program.mvp, 1, gl::FALSE, &mvp[0][0]);

        gl::BindVertexArray(vao);
        gl::DrawElements(gl::TRIANGLES, INDICES.len() as i32, gl::UNSIGNED_INT, ptr::null());
        gl::BindVertexArray(0);
    }
}

pub fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    let (mut win, evts) = glfw.create_window(WIDTH, HEIGHT, TITLE, glfw::WindowMode::Windowed).unwrap();
    win.make_current();
    win.set_framebuffer_size_polling(true);
    win.set_key_polling(true);
    let (vao, program) = setup_gl(&mut win);
    let mut camera = Camera{
        position: vec3(0.0, 0.0, 5.0),
        direction: vec3(0.0, 0.0, -1.0),
        keys: HashSet::new(),
    };
    while !win.should_close() {
        process_events(&evts, &mut win, &mut camera);
        render(vao, &program, &mut camera);
        win.swap_buffers();
        glfw.poll_events();
    }
}

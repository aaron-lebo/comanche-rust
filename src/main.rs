extern crate cgmath;
extern crate gl;
extern crate glfw;

use cgmath::{Matrix4, Point3, SquareMatrix, vec3};
use gl::types::*;
use self::glfw::Context;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::str;

const TITLE: &str = "comanche";

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const VERTS: [f32; 24] = [
    -0.5,  0.5,  0.5,
    -0.5, -0.5,  0.5,
     0.5, -0.5,  0.5,
     0.5,  0.5,  0.5,
    -0.5,  0.5, -0.5,
    -0.5, -0.5, -0.5,
     0.5, -0.5, -0.5,
     0.5,  0.5, -0.5,
];
const INDICES: [i32; 36] = [
    0, 1, 2, 2, 3, 0, // +z
    4, 5, 6, 6, 7, 4, // -z
    4, 0, 3, 3, 7, 4, // +y
    5, 1, 2, 2, 6, 5, // -y
    3, 2, 6, 6, 7, 3, // +x
    0, 1, 5, 5, 4, 0, // -x
];

struct ShaderProgram {
    program: GLuint,
    mvp: GLint,
}

unsafe fn check_status(item: GLuint, kind: &str) {
    let mut ok = gl::FALSE as GLint;
    let get_info = if kind == "program" {
        gl::GetProgramiv(item, gl::LINK_STATUS, &mut ok);
        gl::GetProgramInfoLog
    } else {
        gl::GetShaderiv(item, gl::COMPILE_STATUS, &mut ok);
        gl::GetShaderInfoLog
    };
    if ok != gl::TRUE as GLint {
        let mut info = Vec::with_capacity(512);
        info.set_len(511);
        get_info(item, 512, ptr::null_mut(), info.as_mut_ptr() as *mut GLchar);
        println!("ERROR: {} compilation failed\n{}", kind, str::from_utf8(&info).unwrap());
    }
}

unsafe fn compile_shader(shader: GLenum, src: &str) -> GLuint {
    let shader = gl::CreateShader(shader);
    let c_str = CString::new(src.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
    gl::CompileShader(shader);
    check_status(shader, if shader == gl::VERTEX_SHADER { "vertex shader" } else { "fragment shader" });
    shader
}

fn setup_gl() -> (ShaderProgram, GLuint) {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Viewport(0, 0, WIDTH as i32, HEIGHT as i32);
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
        gl::LogicOp(gl::INVERT);

        let program = {
            let p = gl::CreateProgram();
            let vs = compile_shader(gl::VERTEX_SHADER, include_str!("block_vert.glsl"));
            let fs = compile_shader(gl::FRAGMENT_SHADER, include_str!("block_frag.glsl"));
            gl::AttachShader(p, vs);
            gl::AttachShader(p, fs);
            gl::LinkProgram(p);
            check_status(p, "program");
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
            ShaderProgram{
                program: p,
                mvp: gl::GetUniformLocation(p, CString::new("mvp").unwrap().as_ptr()),
            }
        };

        let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (VERTS.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &VERTS[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                       (INDICES.len() * mem::size_of::<GLint>()) as GLsizeiptr,
                       &INDICES[0] as *const i32 as *const c_void,
                       gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * mem::size_of::<GLfloat>() as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::BindVertexArray(0);
        (program, vao)
    }
}

fn render(program: &ShaderProgram, vao: GLuint) {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        gl::UseProgram(program.program);
        let projection = cgmath::perspective(cgmath::Deg(45.0), 4.0 / 3.0, 0.1, 100.0);
        let view = Matrix4::look_at(Point3::new(4.0, 3.0, -3.0), Point3::new(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
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
    win.set_key_polling(true);
    win.make_current();
    win.set_framebuffer_size_polling(true);

    gl::load_with(|sym| win.get_proc_address(sym) as *const _);
    let (program, vao) = setup_gl();
    while !win.should_close() {
        use self::glfw::{Key, Action};
        for (_, evt) in glfw::flush_messages(&evts) {
            match evt {
                glfw::WindowEvent::FramebufferSize(w, h) => unsafe { gl::Viewport(0, 0, w, h) },
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => win.set_should_close(true),
                _ => (),
            }
        }

        render(&program, vao);
        win.swap_buffers();
        glfw.poll_events();
    }
}

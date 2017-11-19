extern crate gl;
extern crate glfw;

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
    let string = CString::new(src.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &string.as_ptr(), ptr::null());
    gl::CompileShader(shader);
    check_status(shader, if shader == gl::VERTEX_SHADER {"vertex shader"} else {"fragment shader"});
    shader
}

fn setup_gl() -> (GLuint, GLuint) {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Viewport(0, 0, WIDTH as i32, HEIGHT as i32);
        //gl::Enable(gl::CULL_FACE);
        //gl::Enable(gl::DEPTH_TEST);
        gl::LogicOp(gl::INVERT);

        const VERT: &str = r#"
            #version 330 core
            layout (location = 0) in vec3 pos;
            void main() {
                gl_Position = vec4(pos, 1.0);
            }"#;
        const FRAG: &str = r#"
            #version 330 core
            out vec4 color;
            void main() {
                color = vec4(0.0f, 0.0f, 1.0f, 1.0f);
            }"#;
        let pro = gl::CreateProgram();
        let vert = compile_shader(gl::VERTEX_SHADER, VERT);
        let frag = compile_shader(gl::FRAGMENT_SHADER, FRAG);
        gl::AttachShader(pro, vert);
        gl::AttachShader(pro, frag);
        gl::LinkProgram(pro);
        check_status(pro, "program");
        gl::DeleteShader(vert);
        gl::DeleteShader(frag);

        const VERTS: [f32; 12] = [
             0.5,  0.5, 0.0,
             0.5, -0.5, 0.0,
            -0.5, -0.5, 0.0,
            -0.5,  0.5, 0.0];
        const INDICES: [i32; 6] = [
            0, 1, 3,
            1, 2, 3];
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
        (pro, vao)
    }
}

fn render(program: GLuint, vao: GLuint) {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::UseProgram(program);

        gl::BindVertexArray(vao);
        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
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
                glfw::WindowEvent::FramebufferSize(w, h) => unsafe { gl::Viewport(0, 0, w, h) }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => win.set_should_close(true),
                _ => ()
            }
        }

        render(program, vao);
        win.swap_buffers();
        glfw.poll_events();
    }
}

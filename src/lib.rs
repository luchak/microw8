use std::io::prelude::*;
use std::path::Path;
use std::{fs::File, time::Instant};

use anyhow::Result;
use minifb::{Key, Window, WindowOptions};
use wasmtime::{
    Engine, GlobalType, Memory, MemoryType, Module, Mutability, Store, TypedFunc, ValType,
};

static GAMEPAD_KEYS: &'static [Key] = &[Key::Up, Key::Down, Key::Left, Key::Right, Key::Z, Key::X, Key::A, Key::S];

pub struct MicroW8 {
    engine: Engine,
    loader_module: Module,
    window: Window,
    window_buffer: Vec<u32>,
    instance: Option<UW8Instance>,
}

struct UW8Instance {
    store: Store<()>,
    memory: Memory,
    end_frame: TypedFunc<(), ()>,
    update: TypedFunc<(), ()>,
    start_time: Instant,
}

impl MicroW8 {
    pub fn new() -> Result<MicroW8> {
        let engine = wasmtime::Engine::default();

        let loader_module =
            wasmtime::Module::new(&engine, include_bytes!("../platform/bin/loader.wasm"))?;

        let mut options = WindowOptions::default();
        options.scale = minifb::Scale::X2;
        options.scale_mode = minifb::ScaleMode::AspectRatioStretch;
        options.resize = true;
        let mut window = Window::new("MicroW8", 320, 240, options)?;
        window.limit_update_rate(Some(std::time::Duration::from_micros(16666)));

        Ok(MicroW8 {
            engine,
            loader_module,
            window,
            window_buffer: vec![0u32; 320 * 240],
            instance: None,
        })
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    fn reset(&mut self) {
        self.instance = None;
        for v in &mut self.window_buffer {
            *v = 0;
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.reset();

        let mut module = vec![];
        File::open(path)?.read_to_end(&mut module)?;
        self.load_from_memory(&module)
    }

    pub fn load_from_memory(&mut self, module: &[u8]) -> Result<()> {
        self.reset();

        let mut store = wasmtime::Store::new(&self.engine, ());

        let memory = wasmtime::Memory::new(&mut store, MemoryType::new(4, Some(4)))?;

        let mut linker = wasmtime::Linker::new(&self.engine);
        linker.define("env", "memory", memory.clone())?;

        let loader_instance = linker.instantiate(&mut store, &self.loader_module)?;
        let load_uw8 = loader_instance.get_typed_func::<i32, i32, _>(&mut store, "load_uw8")?;

        let platform_data = include_bytes!("../platform/bin/platform.uw8");
        memory.data_mut(&mut store)[..platform_data.len()].copy_from_slice(platform_data);
        let platform_length =
            load_uw8.call(&mut store, platform_data.len() as i32)? as u32 as usize;
        let platform_module =
            wasmtime::Module::new(&self.engine, &memory.data(&store)[..platform_length])?;

        memory.data_mut(&mut store)[..module.len()].copy_from_slice(module);
        let module_length = load_uw8.call(&mut store, module.len() as i32)? as u32 as usize;
        let module = wasmtime::Module::new(&self.engine, &memory.data(&store)[..module_length])?;

        linker.func_wrap("env", "acos", |v: f32| v.acos())?;
        linker.func_wrap("env", "asin", |v: f32| v.asin())?;
        linker.func_wrap("env", "atan", |v: f32| v.atan())?;
        linker.func_wrap("env", "atan2", |x: f32, y: f32| x.atan2(y))?;
        linker.func_wrap("env", "cos", |v: f32| v.cos())?;
        linker.func_wrap("env", "exp", |v: f32| v.exp())?;
        linker.func_wrap("env", "log", |v: f32| v.ln())?;
        linker.func_wrap("env", "sin", |v: f32| v.sin())?;
        linker.func_wrap("env", "tan", |v: f32| v.tan())?;
        linker.func_wrap("env", "pow", |a: f32, b: f32| a.powf(b))?;
        for i in 9..64 {
            linker.func_wrap("env", &format!("reserved{}", i), || {})?;
        }
        for i in 0..16 {
            linker.define(
                "env",
                &format!("g_reserved{}", i),
                wasmtime::Global::new(
                    &mut store,
                    GlobalType::new(ValType::I32, Mutability::Const),
                    0.into(),
                )?,
            )?;
        }

        let platform_instance = linker.instantiate(&mut store, &platform_module)?;

        for export in platform_instance.exports(&mut store) {
            linker.define(
                "env",
                export.name(),
                export
                    .into_func()
                    .expect("platform surely only exports functions"),
            )?;
        }

        let instance = linker.instantiate(&mut store, &module)?;
        let end_frame = platform_instance.get_typed_func::<(), (), _>(&mut store, "endFrame")?;
        let update = instance.get_typed_func::<(), (), _>(&mut store, "upd")?;

        self.instance = Some(UW8Instance {
            store,
            memory,
            end_frame,
            update,
            start_time: Instant::now(),
        });

        Ok(())
    }

    pub fn run_frame(&mut self) -> Result<()> {
        if let Some(mut instance) = self.instance.take() {
            {
                let time = instance.start_time.elapsed().as_millis() as i32;
                let mut gamepad: u32 = 0;
                for key in self.window.get_keys().unwrap_or(Vec::new()) {
                    if let Some(index) = GAMEPAD_KEYS.iter().enumerate().find(|(_, &k)| k == key).map(|(i, _)| i) {
                        gamepad |= 1 << index;
                    }
                }

                let mem = instance.memory.data_mut(&mut instance.store);
                mem[64..68].copy_from_slice(&time.to_le_bytes());
                mem[68..72].copy_from_slice(&gamepad.to_le_bytes());
            }

            instance.update.call(&mut instance.store, ())?;
            instance.end_frame.call(&mut instance.store, ())?;

            let framebuffer = &instance.memory.data(&instance.store)[120..];
            let palette = &framebuffer[320 * 240..];
            for i in 0..320 * 240 {
                let offset = framebuffer[i] as usize * 4;
                self.window_buffer[i] = 0xff000000
                    | ((palette[offset + 0] as u32) << 16)
                    | ((palette[offset + 1] as u32) << 8)
                    | palette[offset + 2] as u32;
            }

            self.instance = Some(instance);
        }

        self.window
            .update_with_buffer(&self.window_buffer, 320, 240)?;

        Ok(())
    }
}
use eframe::egui;

use crate::backend::protocol::BackEnd;
use crate::frontend;
use frontend::auth::AuthFlow;
use frontend::main_view::{MainScreen, types::{UiChat, UiTopic}};

pub fn run() -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default() 
            .with_maximized(true)
            .with_resizable(true),
        ..Default::default()
    };
    
    eframe::run_native("EmbedSearch", native_options, Box::new(|c| {Ok(Box::new(App::new(c)))}))?;
    Ok(())
}


pub enum Screen {
    Auth(AuthFlow),
    Main(MainScreen),
} 


pub struct App {
    screen: Screen,
    backend: BackEnd,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let backend = BackEnd::initialize();
        let screen = Screen::Auth(AuthFlow::init());
            
        Self {
            backend,
            screen,
        }
    }
}

impl eframe::App for App {
   fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
       match &mut self.screen {
           Screen::Auth(flow) => {
               if let Some(init_response) = flow.update(&mut self.backend) {
                   self.screen = Screen::Main(MainScreen::from_init_resp(init_response));
                   return;
               }
               flow.draw(ui);
           },
           Screen::Main(screen) => {
               screen.update(&self.backend);
               screen.draw(ui);
           }
       }
   }
}


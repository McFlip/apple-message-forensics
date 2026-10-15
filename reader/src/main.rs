use eframe::egui;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

enum ServerEvent {
    Ready(String),
    BrowserError(String),
    Failed(String),
    Stopped,
}

enum ServerCommand {
    Shutdown,
}

struct ReaderApp {
    status: String,
    detail: Option<String>,
    events: Receiver<ServerEvent>,
    commands: Sender<ServerCommand>,
    worker_finished: bool,
    shutting_down: bool,
    close_when_stopped: bool,
}

impl ReaderApp {
    fn new(report_dir: PathBuf, server_executable: PathBuf) -> Self {
        let (event_sender, events) = mpsc::channel();
        let (commands, command_receiver) = mpsc::channel();
        thread::spawn(move || {
            run_server(
                report_dir,
                server_executable,
                command_receiver,
                event_sender,
            );
        });

        Self {
            status: "Launching server. Please stand by...".to_owned(),
            detail: None,
            events,
            commands,
            worker_finished: false,
            shutting_down: false,
            close_when_stopped: false,
        }
    }

    fn begin_shutdown(&mut self, ctx: &egui::Context) {
        if self.shutting_down {
            return;
        }

        if self.worker_finished {
            self.close_when_stopped = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.shutting_down = true;
        self.status = "Shutting down the server. Please wait.".to_owned();
        let _ = self.commands.send(ServerCommand::Shutdown);
        ctx.request_repaint();
    }
}

impl eframe::App for ReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|input| input.viewport().close_requested()) && !self.close_when_stopped {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.begin_shutdown(ctx);
        }

        loop {
            match self.events.try_recv() {
                Ok(ServerEvent::Ready(url)) => {
                    if !self.shutting_down {
                        self.status = format!("Report ready at {url}");
                    }
                }
                Ok(ServerEvent::BrowserError(error)) => self.detail = Some(error),
                Ok(ServerEvent::Failed(error)) => {
                    self.worker_finished = true;
                    self.status = format!("Unable to launch the report server: {error}");
                    self.detail = None;
                    if self.close_when_stopped || self.shutting_down {
                        self.close_when_stopped = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
                Ok(ServerEvent::Stopped) => {
                    self.worker_finished = true;
                    self.close_when_stopped = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(24.0);
                ui.label(&self.status);
                if let Some(detail) = &self.detail {
                    ui.label(detail);
                }
                if !self.shutting_down {
                    ui.add_space(16.0);
                    let exit =
                        egui::Button::new(egui::RichText::new("Exit").color(egui::Color32::WHITE))
                            .fill(egui::Color32::RED);
                    if ui.add(exit).clicked() {
                        self.begin_shutdown(ctx);
                    }
                }
            });
        });

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn run_server(
    report_dir: PathBuf,
    server_executable: PathBuf,
    commands: Receiver<ServerCommand>,
    events: Sender<ServerEvent>,
) {
    let mut child = match Command::new(&server_executable)
        .arg(&report_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            let _ = events.send(ServerEvent::Failed(format!(
                "could not start {}: {error}",
                server_executable.display()
            )));
            return;
        }
    };

    let url = match read_server_url(&mut child) {
        Ok(url) => url,
        Err(error) => {
            let error = match child.wait() {
                Ok(_) => error,
                Err(wait_error) => {
                    format!("{error}; could not wait for report server: {wait_error}")
                }
            };
            let _ = events.send(ServerEvent::Failed(error));
            return;
        }
    };

    let _ = events.send(ServerEvent::Ready(url.clone()));
    let shutting_down = !matches!(commands.try_recv(), Err(TryRecvError::Empty));
    if !shutting_down {
        if let Err(error) = webbrowser::open(&url) {
            let _ = events.send(ServerEvent::BrowserError(format!(
                "Could not open the default browser: {error}"
            )));
        }
        let _ = commands.recv();
    }

    drop(child.stdin.take());
    let result = child.wait();
    if let Err(error) = result {
        let _ = events.send(ServerEvent::Failed(format!(
            "could not wait for the report server to stop: {error}"
        )));
    }
    let _ = events.send(ServerEvent::Stopped);
}

fn read_server_url(child: &mut Child) -> Result<String, String> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "report server stdout was unavailable".to_owned())?;
    let mut line = String::new();
    BufReader::new(stdout)
        .read_line(&mut line)
        .map_err(|error| format!("could not read report server address: {error}"))?;
    let url = line.trim();
    if url.is_empty() {
        return Err("report server exited before reporting its address".to_owned());
    }
    Ok(url.to_owned())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executable = std::env::current_exe()?;
    let executable_dir = executable
        .parent()
        .ok_or_else(|| std::io::Error::other("reader executable has no parent directory"))?;
    let report_dir = if executable_dir.join("index.html").is_file() {
        executable_dir.to_owned()
    } else {
        std::env::current_dir()?.join("output/report")
    };
    let server_executable = executable_dir.join(if cfg!(windows) {
        "report-server.exe"
    } else {
        "report-server"
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([520.0, 160.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Report Reader",
        options,
        Box::new(move |_creation_context| {
            Ok(Box::new(ReaderApp::new(report_dir, server_executable)))
        }),
    )?;
    Ok(())
}

pub mod main_window;

use chrono::Local;
use gtk::{prelude::*, ApplicationWindow, Builder};
use patk_bittorrent_client::{
    client::client_side::ClientSide, config::Config, logging::logger::Logger,
    server::server_side::ServerSide, utils,
};
use std::{borrow::Borrow, sync::mpsc, thread};

fn build_main_window(application: &gtk::Application, builder: Builder) -> ApplicationWindow {
    let window: ApplicationWindow = builder.object("main_window").expect("problema");
    window.set_application(Some(application));
    window
}

fn build_add_torrent_button(builder: Builder, dialog: gtk::Dialog) {
    let add_button: gtk::Button = builder.object("add_button1").expect("no add button");

    add_button.connect_clicked(move |_| {
        dialog.show_all();
    });
}

fn build_config_button(builder: Builder, dialog: gtk::Dialog) {
    let config_button: gtk::Button = builder.object("config_button1").expect("no config button");

    config_button.connect_clicked(move |_| {
        dialog.show_all();
    });
}

fn build_file_chooser_text_view(builder: Builder) -> gtk::Entry {
    let torrent_url: gtk::Entry = builder
        .object("file_chooser_dialog_text")
        .expect("no Entry");
    torrent_url
}

fn build_file_chooser_window(builder: Builder) -> gtk::Dialog {
    let dialog: gtk::Dialog = builder
        .object("file_choosing_dialog")
        .expect("no dialog window");
    dialog.connect_delete_event(|dialog, _| {
        dialog.hide();
        gtk::Inhibit(true)
    });
    dialog
}

fn build_config_window(builder: Builder) -> gtk::Dialog {
    let dialog: gtk::Dialog = builder.object("config_dialog").expect("no dialog window");
    dialog.connect_delete_event(|dialog, _| {
        dialog.hide();
        gtk::Inhibit(true)
    });
    dialog
}

fn build_accept_url_button(
    builder: Builder,
    text: gtk::Entry,
    dialog: gtk::Dialog,
    grid: gtk::Grid,
    sender: mpsc::Sender<String>,
) {
    let add_button: gtk::Button = builder
        .object("file_chooser_dialog_accept_button")
        .expect("no accept button");

    add_button.connect_clicked(move |_| {
        dialog.close();
        let _ = sender.send(text.buffer().text());
        add_row_to_grid(grid.clone(), &text.buffer().text());
    });
}

fn build_cancel_button(builder: Builder, dialog: gtk::Dialog, id: &str) {
    let cancel_button: gtk::Button = builder.object(id).expect("no cancel button");

    cancel_button.connect_clicked(move |_| {
        dialog.close();
    });
}

fn get_torrent_grid(builder: Builder) -> gtk::Grid {
    let grid: gtk::Grid = builder.object("torrent_table").expect("no table");
    grid
}

fn add_row_to_grid(grid: gtk::Grid, mut name: &str) {
    match name.split('/').last() {
        Some(new_name) => name = new_name,
        None => todo!(),
    }
    let file_name = gtk::Label::new(Some(name));
    let status = gtk::Label::new(Some("Downloading"));
    file_name.set_margin_bottom(10);
    file_name.set_widget_name(name);
    status.set_margin_bottom(10);
    let mut top = grid.children().len().try_into().unwrap();
    top -= 2;
    grid.attach(file_name.borrow(), 0, top, 1, 1);
    grid.attach(status.borrow(), 1, top, 1, 1);
    file_name.show();
    status.show();
}

fn build_ui(application: &gtk::Application, sender: mpsc::Sender<String>) -> ApplicationWindow {
    let glade1_src = include_str!("view1.glade");
    let builder = Builder::from_string(glade1_src);
    let window = build_main_window(application, builder.clone());
    let torrent_url: gtk::Entry = build_file_chooser_text_view(builder.clone());
    let add_torrent_dialog: gtk::Dialog = build_file_chooser_window(builder.clone());
    build_add_torrent_button(builder.clone(), add_torrent_dialog.clone());
    build_cancel_button(
        builder.clone(),
        add_torrent_dialog.clone(),
        "file_chooser_dialog_cancel_button",
    );
    let config_dialog: gtk::Dialog = build_config_window(builder.clone());
    build_config_button(builder.clone(), config_dialog.clone());
    build_cancel_button(
        builder.clone(),
        config_dialog,
        "config_dialog_cancel_button",
    );
    let grid = get_torrent_grid(builder.clone());
    build_accept_url_button(builder, torrent_url, add_torrent_dialog, grid, sender);

    window
}

//cargo run --package patk_bittorrent_client --bin ui_test --all-features

fn start_client_worker(receiver: mpsc::Receiver<String>, mut client: ClientSide) {
    let _ = thread::spawn(move || loop {
        if let Ok(user_input) = receiver.recv() {
            let torrent_path = vec![user_input];
            let _ = client.load_torrents(torrent_path);
        };
    });
}

fn main() -> Result<(), String> {
    let application = gtk::Application::new(
        Some(&format!(
            "com.Panic_at_the_kernel.ui-{}",
            Local::now().timestamp()
        )),
        Default::default(),
    );

    let config = Config::new()?;
    let logger = Logger::new(config.log_path())?;

    let mut client = ClientSide::new(&config, logger.handle());
    let mut server = ServerSide::new(client.get_id(), &config, logger.handle());

    let (notif_tx, notif_rx) = mpsc::channel();
    server.init(notif_tx.clone(), notif_rx)?;
    client.init(notif_tx)?;

    let log_peer_id = format!(
        "Client Peer ID: {}",
        utils::bytes_to_string(&client.get_id())?
    );
    let (path_tx, path_rx) = mpsc::channel::<String>();

    start_client_worker(path_rx, client);

    application.connect_activate(move |app| {
        let window = build_ui(app, path_tx.clone());
        window.show();
    });
    logger.handle().log(&log_peer_id)?;
    let code = application.run();
    std::process::exit(code)
}

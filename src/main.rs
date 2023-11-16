#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::{thread, time::Duration};
use egui::RichText;
use enigo::{Enigo, Key, KeyboardControllable};
use device_query_revamped::{DeviceEvents, DeviceState, Keycode};
use selection::get_text;
use eframe::egui;
use std::sync::mpsc::{self, Sender, Receiver};
use std::process;

mod translation_and_api;
use crate::translation_and_api::{get_api_key, get_translation, is_api_key_valid, save_api_key};

struct AppVariables {
    enigo: Enigo,
    api_key: String,
}

fn main() {


    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    let app_variables = AppVariables {
        enigo: Enigo::new(),
        api_key: get_api_key(),
    };

    if !is_api_key_valid(&app_variables.api_key) {
        invalid_api_key_gui();
    }


    let mut auto_translation = false;
    let mut click_translation = false;

    let (tx, rx): (Sender<(bool, bool)>, Receiver<(bool, bool)>) = mpsc::channel();
    let (mouse_input_tx, mouse_input_rx): (Sender<()>, Receiver<()>) = mpsc::channel();
    let (keyboard_input_tx, keyboard_input_rx): (Sender<()>, Receiver<()>) = mpsc::channel();
    let (word_tx, word_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    thread::spawn(move || main_loop(rx, mouse_input_rx, keyboard_input_rx, word_tx, app_variables));

    let device_state = DeviceState::new();
    let _guard = device_state.on_mouse_up(move |button| {
        if let 1 = button {
            let _ = mouse_input_tx.send(());
        }
    });
    let _guard = device_state.on_key_down(move |key| {
        match key {
            Keycode::F11 => {
                let _ = keyboard_input_tx.send(());
            },
            _ => ()
        }
    });

    let mut current_word = String::from("Pressez F11 pour traduire le mot du curseur.");

    let _ = eframe::run_simple_native("Übersetzer", options, move |ctx, _frame| {
        ctx.request_repaint_after(Duration::from_millis(100));
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Übersetzer").size(35.0));
                ui.add_space(22.0);
                ui.checkbox(&mut auto_translation, RichText::new("Traduction automatique").size(15.0));
                ui.add_space(5.0);
                ui.checkbox(&mut click_translation, RichText::new("Cliquer pour traduire").size(15.0));
                ui.add_space(10.0);
                if ui.button("Confirmer les préférences").clicked() {
                    let _ = tx.send((auto_translation.clone(), click_translation.clone()));
                }
                ui.add_space(20.0);
                ui.label(RichText::new(format!("{}", (if let Ok(word) = word_rx.try_recv() { current_word = word; &mut current_word } else {&mut current_word }))).size(25.0));
    
            });
            
        });
    });
    

    let mut enigo = Enigo::new();
    enigo.key_up(Key::Control);
    enigo.key_up(Key::Shift);
}

fn main_loop(rx: Receiver<(bool, bool)>, mouse_input_rx: Receiver<()>, keyboard_input_rx: Receiver<()>, word_tx: Sender<String>, mut app_variables: AppVariables) {

    if !is_api_key_valid(&app_variables.api_key) {
        println!("Please provide a valid api key.");
        return
    }

    let mut auto_translation: bool = false;
    let mut click_translation: bool = false;

    // Main loop
    loop {

        if let Ok(received) = rx.try_recv() {
            auto_translation = received.0;
            click_translation = received.1;
            let _ = mouse_input_rx.try_recv();
        }

        if let Ok(_) = mouse_input_rx.try_recv() {
            if click_translation {
                translate(&auto_translation, &app_variables.api_key, &mut app_variables.enigo, &word_tx);
            }
        }

        if let Ok(_) = keyboard_input_rx.try_recv() {
            translate(&auto_translation, &app_variables.api_key, &mut app_variables.enigo, &word_tx);
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn invalid_api_key_gui() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    let mut new_api_key = "".to_owned();
    let mut help_text = "".to_owned();

    let _ = eframe::run_simple_native("Setup", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.vertical_centered(|ui| {
                ui.heading("Veuillez entrer une clé valide.");
                ui.text_edit_singleline(&mut new_api_key);
                if ui.button("Confirmer").clicked() {
                    if is_api_key_valid(&new_api_key) {
                        save_api_key(&new_api_key);
                        help_text = "Clé valide !\nVous pouvez maintenant fermer cette fenêtre et redémarrer l'app."
                            .to_owned();
                    }
                    else {
                        new_api_key = "Cette clé n'est pas valide.".to_owned();
                        help_text =
                            "Pour trouver votre clé, rendez-vous sur https://www.deepl.com et connectez-vous ou créez un compte.
                            \nTrouvez ensuite votre 'Clé d'authentification pour l'API de DeepL' sous compte -> compte ou à partir du lien suivant: https://www.deepl.com/account/summary."
                            .to_owned();
                    }
                }
                ui.label(RichText::new(format!("{}", help_text)).size(14.0));
            });
        });
    });
    process::exit(0);
}

fn translate(auto_translation: &bool, api_key: &String, enigo: &mut Enigo, word_tx: &Sender<String>) {
    let translation = get_translation(&api_key , &get_word(enigo));
    if *auto_translation {
        auto_translate(&translation, enigo);
    }
    let _ = word_tx.send(translation);
}

fn get_word(enigo: &mut Enigo) -> String {

    enigo.key_down(Key::Control);
    enigo.key_click(Key::RightArrow);
    enigo.key_down(Key::Shift);
    enigo.key_click(Key::LeftArrow);
    enigo.key_up(Key::Control);
    enigo.key_up(Key::Shift);
    
    get_text()
}

fn auto_translate(translation: &String, enigo: &mut Enigo) {
    enigo.key_click(Key::RightArrow);
    let mut text: String = String::from("(");
    text.push_str(&translation);
    text += ")";
    enigo.key_sequence(&text);
}
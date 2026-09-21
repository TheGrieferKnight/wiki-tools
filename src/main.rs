use std::{env, thread};

use wiki_tools::{AppError, SearchField, build_magic_graph, setup_csv};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

#[cfg(debug_assertions)]
async fn run() -> Result<(), AppError> {
    setup_csv().await?;

    let stack_size = 8 * 1024 * 1024; // Set stack size to 8mb since some Spells overflow the 1mb standard stack size limit *COUGH* Beast Claw *COUGH COUGH* Nail Volley

    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        return Err(AppError::UnexpectedArgs { args });
    }

    if args.len() < 2 {
        return Err(AppError::NoArgs);
    }

    let spell_name = args[1].clone();

    let handle = thread::Builder::new()
        .name("Large Stack Worker for test".to_owned())
        .stack_size(stack_size)
        .spawn(move || -> Result<(), AppError> {
            let graph = build_magic_graph(SearchField::Name(spell_name))?;
            let dot = graph.to_dot();

            std::fs::write("magic.dot", dot)
                .map_err(AppError::from)?;

            Ok(())
        })
        .map_err(AppError::from)?;

    handle
        .join()
        .map_err(|panic_payload| AppError::WorkerPanicked {
            message: panic_message(panic_payload),
        })??;

    println!("Spell has been found and processed.");

    Ok(())
}

#[cfg(not(debug_assertions))]
async fn run() -> Result<(), AppError> {
    setup_csv().await?;

    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        return Err(AppError::UnexpectedArgs { args });
    }

    if args.len() < 2 {
        return Err(AppError::NoArgs);
    }

    let spell_name = args[1].clone();

    let graph = build_magic_graph(SearchField::Name(spell_name))?;
    let dot = graph.to_dot();

    std::fs::write("magic.dot", dot).map_err(AppError::from)?;


    println!("Spell has been found and processed.");

    Ok(())
}

#[allow(unused)]
fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "worker thread panicked with an unknown payload".to_owned()
    }
}
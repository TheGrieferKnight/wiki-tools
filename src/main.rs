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

    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", ".\\tools.\\graphviz\\windows-x64\\dot.exe -Tsvg magic.dot -o magic.svg"])
            .output()
            .expect("failed to execute process")
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg("dot -Tsvg magic.dot -o magic.svg")
            .output()
            .expect("failed to execute process")
    };

    if !output.status.success() {
        return Err(AppError::ExitStatusError { exit_status: output.status })
    }

    println!("Successfully converted magic.dot to magic.svg");

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

    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", ".\\tools\\graphviz\\windows-x64\\dot.exe -Tsvg magic.dot -o magic.svg"])
            .output()
            .expect("failed to execute process")
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg("dot -Tsvg magic.dot -o magic.svg")
            .output()
            .expect("failed to execute process")
    };

    if !output.status.success() {
        return Err(AppError::ExitStatusError { exit_status: output.status })
    }

    println!("Successfully converted magic.dot to magic.svg");

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
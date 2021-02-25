use std::fs;

use lib_ruby_parser::Parser;
use routes::{parse_routes, Request};
use ruby_parser::{parse_file, Method, RubyFile};
use walkdir::{DirEntry, WalkDir};

pub mod params;
mod parser_parser;
pub mod routes;
mod ruby_parser;

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn parse_files(path: &str) -> Result<Vec<RubyFile>, Box<dyn std::error::Error>> {
    let mut errors = Vec::new();
    let mut file_count = 0;
    let mut results = Vec::new();
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
    {
        let f = entry.path();
        if f.is_file() && f.display().to_string().ends_with(".rb") {
            let parser = Parser::new(&fs::read(entry.path())?, Default::default());
            let result = parse_file(parser.do_parse().ast.unwrap());
            if result.is_ok() {
                results.push(result.unwrap());
            } else {
                errors.push((f.display().to_string(), result.err()));
            }
        }
        file_count += 1;
    }

    if !errors.is_empty() {
        println!(
            "Got {} errors out of a total of {} files",
            errors.len(),
            file_count
        );

        println!("{:?}", errors);
    }

    Ok(results)
}

fn get_name(module: &str, controller: &str) -> String {
    let mut name = module.to_lowercase();

    if !name.is_empty() {
        name += "/";
    }

    let mut first = true;
    let controller_name: Vec<&str> = controller.split("::").collect();
    if controller_name.len() == 2 {
        name += &(controller_name[0].to_lowercase() + "/");
        for c in controller_name[1].replace("Controller", "").chars() {
            if first {
                name += &c.to_lowercase().to_string();
                first = !first;
            } else if c.is_uppercase() {
                name += "_";
                name += &c.to_lowercase().to_string();
            } else {
                name += &c.to_string();
            }
        }
    } else {
        for c in controller_name[0].replace("Controller", "").chars() {
            if first {
                name += &c.to_lowercase().to_string();
                first = !first;
            } else if c.is_uppercase() {
                name += "_";
                name += &c.to_lowercase().to_string();
            } else {
                name += &c.to_string();
            }
        }
    }

    name
}

fn search_in_routes(
    name: &str,
    routes: &Vec<Request>,
    methods: &Vec<Method>,
) -> Result<(), String> {
    let mut found = false;
    for route in routes {
        if route.controller == name {
            for method in methods {
                if method.name == route.action {
                    println!("{} {}", route, method);
                    found = true;
                }
            }
        }
    }
    if !found {
        Err("unable to find method on controller".to_string())
    } else {
        Ok(())
    }
}

pub fn compute(controller_path: &str, routes_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: abstract these out so unit tests can written... ah more work but will help
    let routes_file = fs::read_to_string(routes_file);
    if let Err(e) = routes_file{
        Err(format!("Error in reading routes file: {}", e.to_string()))?
    }else{
        let files = parse_files(controller_path)?;
        let routes = parse_routes(&routes_file.unwrap())?;
    
        for file in &files {
            for module in &file.modules {
                for controller in &module.classes {
                    if search_in_routes(
                        &get_name(&module.name, &controller.name),
                        &routes,
                        &controller.methods,
                    )
                    .is_err()
                    {
                        println!(
                            "unable to find controller for {} {} -- {}",
                            module.name,
                            controller.name,
                            &get_name(&module.name, &controller.name)
                        );
                    }
                }
            }
    
            for controller in &file.controllers {
                if search_in_routes(
                    &get_name("", &controller.name),
                    &routes,
                    &controller.methods,
                )
                .is_err()
                {
                    println!(
                        "unable to find controller for {} {} -- {}",
                        "",
                        controller.name,
                        &get_name("", &controller.name)
                    );
                }
            }
        }
    }
    

    Ok(())
}

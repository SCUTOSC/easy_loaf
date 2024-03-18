use minigrep::Config;
use std::env;
use std::process;
use minigrep::run;
fn run() {
    let config=Config::new(env::args()).unwrap_or_else(|err|{
            eprintln!("Problems parsing arguments: {}",err);
            process::exit(1);
    });
    if let Err(e)=minigrep::run(config){
        eprintln!("Application Error: {}",e);
        process::exit(1);
    }    
}


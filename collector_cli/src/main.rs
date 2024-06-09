mod list_parse;
mod args;

use args::*; 
use list_parse::ArtifactListing;
use collector_engine::collect::Collect;
use collector_engine::parser::{YamlParser, YamlArtifact};
use collector_engine::collectvss::CollectVss;
use std::fs::File;
use clap::Parser;
use log::*;
use simplelog::*;
use std::time;
use chrono::Utc;
use sysinfo::System;
use std::panic;

fn custom_panic_hook(){
    panic::set_hook(Box::new(|info| {
        // Check if the panic has a message
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            eprintln!("{}", s);
        } else {
            eprintln!("An unexpected error occurred.");
        }
    }));
}

#[tokio::main]
async fn main(){
    custom_panic_hook();
    // Argument parser
    let args = ArgsCollector::parse();
    let src_string = args.source;
    let dst_string = args.destination;
    let zip_name = args.zip;
    let zip_password = args.pass;
    let get_logging = args.log;
    let verbose = args.verbose;


    if args.command.is_some() {
        let args_unwrap = args.command.unwrap(); 
        match args_unwrap {
            RessourcesCommand::Ressources(listing) => {
                let parser_obj: YamlParser = YamlParser::new(args.path_ressources.clone());
                let listor = parser_obj.get_yaml_file();
                let doc_artifacts: Vec<YamlArtifact> = parser_obj.get_doc_struct(listor).await;
                let load_art_list = ArtifactListing::load(doc_artifacts);
                match listing.command {
                    ListRessources::Targets => {
                        for name in load_art_list.names_pa(){
                            println!("{}",name);
                        }
                        ;return},
                    ListRessources::Groups => {
                        for name in load_art_list.names_gr(){
                            println!("{}",name);
                        }
                        ;return},
                    ListRessources::Categories => {
                        for name in load_art_list.list_categories(){
                            println!("{}",name);
                        }
                        ;return},
                };
            },
        }
    }

    let mut config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .add_filter_ignore_str("collector_engine::collect")
        .build();
    if verbose{     
        config = ConfigBuilder::new()
            .set_time_format_rfc3339()
            .build();
    }
    let get_time = Utc::now().timestamp().to_string();
    let get_hostname = System::host_name().unwrap();
    let name_log_file = format!("collector_{}_{}.log",get_hostname,get_time);
    // logger
    if get_logging {        
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Info,
                config.clone(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                LevelFilter::Info,
                config.clone(),
                File::create(&name_log_file).unwrap(),
            ),
        ]).unwrap();
    }else{
        CombinedLogger::init(vec![
            WriteLogger::new(
                LevelFilter::Info,
                config.clone(),
                File::create(&name_log_file).unwrap(),
            ),
        ]).unwrap();
    }
    
    let now = time::Instant::now();

    info!("{}","=".repeat(50));
    info!("Source of artifact: \"{}\"",src_string);
    info!("Destination of artifact: \"{}\"",dst_string);
    info!("List of ressources collect: {:?}",args.ressources);
    info!("Path of getting ressources files: \"{}\"",args.path_ressources);
    info!("Output file log: \"{}\"",&name_log_file);
    info!("{}","=".repeat(50));


    // Parse yaml files in ressources folder
    info!("Parse yaml ressources files");
    let arg_ressources = args.ressources;
    let mut parser_obj: YamlParser = YamlParser::new(args.path_ressources);
    let listor = parser_obj.get_yaml_file();
    let doc_artifacts: Vec<YamlArtifact> = parser_obj.get_doc_struct(listor).await;
    let list_artifacts: Vec<String> = parser_obj.select_artifact(arg_ressources,doc_artifacts);
    info!("End to parse yaml ressources files");
    
    
    // Start collect
    info!("Start to collect artifact");
    let mut collector_obj = Collect::new(src_string.clone(),dst_string.clone(),list_artifacts.clone()).await;
    let _collector_obj_start = collector_obj.start().await;
    info!("End to collect artifact");

    // Start collect vss
    let if_vss: bool = args.vss;
    if if_vss{
        info!("Start to collect artifact from VSS");
        let vss_obj = CollectVss::new(src_string.clone(),dst_string,list_artifacts.clone());
        vss_obj.collect().await;
        info!("End to collect artifact from vss");
    }

    // zip if need
    if zip_name{
        info!("Start to zip output directory");
        let _result = collector_obj.zip(zip_password).await;
        info!("End to zip output directory");
    }

    let elapsed_time = now.elapsed();
    info!("Running took {} seconds.", elapsed_time.as_secs());

}
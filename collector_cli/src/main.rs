// mod list_parse;
// use list_parse::ArtefactListing;
use collector_engine::collect::Collect;
use collector_engine::parser::{YamlParser, YamlArtefact};
use collector_engine::vss::CollectVss;
use std::fs::File;
use clap::Parser;
use log::*;
use simplelog::*;

use std::time;

/// This is a best and fast artefact collector.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args{
    /// The source of collecting artefact.
    #[arg(short,long, default_value="C:")]
    source: String,

    /// The destination of collecting artefact. 
    #[arg(short,long, default_value=".\\out\\")]
    destination: String,

    /// Ressources selection.
    #[arg(short,long, default_value="All",value_delimiter = ',')]
    ressources: Vec<String>,

    /// The path of artefact ressource collection.
    #[arg(short,long,default_value=".\\ressources\\")]
    path_ressources: String,

    /// Zip output directory.
    #[arg(long="zip")]
    zip_name: Option<String>,

    /// Zip output directory.
    #[arg(long)]
    vss: bool,

    /// Print log output in terminal. (Little bit longer)
    #[arg(long)]
    log: bool,

    /// Verbose log
    #[arg(short,long)]
    verbose: bool,


}

#[tokio::main]
async fn main(){
    // Argument parser
    let args = Args::parse();
    let src_string = args.source;
    let dst_string = args.destination;
    let zip_name = args.zip_name;
    let get_logging = args.log;
    let verbose = args.verbose;

    let mut config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .add_filter_ignore_str("collector_engine::collect")
        .build();
    if verbose{     
        config = ConfigBuilder::new()
            .set_time_format_rfc3339()
            .build();
    }

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
                File::create("collector.log").unwrap(),
            ),
        ]).unwrap();
    }else{
        CombinedLogger::init(vec![
            WriteLogger::new(
                LevelFilter::Info,
                config.clone(),
                File::create("collector.log").unwrap(),
            ),
        ]).unwrap();
    }
    
    let now = time::Instant::now();

    info!("{}","=".repeat(50));
    info!("Source of artefact: \"{}\"",src_string);
    info!("Destination of artefact: \"{}\"",dst_string);
    info!("List of ressources collect: {:?}",args.ressources);
    info!("Path of getting ressources files: \"{}\"",args.path_ressources);
    info!("{}","=".repeat(50));


    // Parse yaml files in ressources folder
    info!("Parse yaml ressources files");
    let arg_ressources = args.ressources;
    let mut parser_obj: YamlParser = YamlParser::new(args.path_ressources);
    let listor = parser_obj.get_yaml_file();
    let doc_artefacts: Vec<YamlArtefact> = parser_obj.get_doc_struct(listor).await;
    let list_artefacts: Vec<String> = parser_obj.select_artefact(arg_ressources,doc_artefacts);
    info!("End to parse yaml ressources files");
    
    
    // Start collect
    info!("Start to collect artefact");
    let collector_obj = Collect::new(&src_string,&dst_string,list_artefacts.clone());
    // let mut collector_obj = Collect::new(&src_string,&dst_string,list_artefacts.clone());
    // let _collector_obj_start = collector_obj.start().await;
    info!("End to collect artefact");

    // Under contruction

    // Start collect vss
    let if_vss: bool = args.vss;
    if if_vss{
        info!("Start to collect artefact from vss");
        let vss_obj = CollectVss::new(&src_string,&dst_string,list_artefacts.clone());
        vss_obj.collect().await;
        // CollectVss::get_list();
        // let res = VSSObj::get_vss_list();
        // println!("{:?}",res);
    }

    // zip if need
    match zip_name{
        Some(name) => {
            info!("Start to zip output directory");
            let _ = collector_obj.zip(name);
            info!("End to zip output directory");
        },
        None => (),
    }

    let elapsed_time = now.elapsed();
    info!("Running took {} seconds.", elapsed_time.as_secs());

}
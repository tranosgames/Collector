use clap::{Parser, Subcommand, Args};

/// This is a best and fast artifact collector.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ArgsCollector{

    #[command(subcommand)]
    pub command: Option<RessourcesCommand>,

    /// The source of collecting artifact.
    #[arg(short,long, default_value="C:\\")]
    pub source: String,

    /// The destination of collecting artifact. 
    #[arg(short,long, default_value=".\\out\\")]
    pub destination: String,

    /// Ressources selection.
    /// You can list with "ressources" command.
    /// Exemple: MFT,Prefetch,EVTX
    #[arg(short,long, default_value="All",value_delimiter = ',')]
    pub ressources: Vec<String>,

    /// Path to artifact resources.
    #[arg(short,long,default_value=".\\ressources\\")]
    pub path_ressources: String,

    /// Zip the output directory.
    #[arg(long)]
    pub zip: bool,

    /// Set zip password.
    #[arg(long)]
    pub pass: Option<String>,

    /// Collect from vss. (longer)
    #[arg(long)]
    pub vss: bool,

    /// Print log output in terminal. (longer)
    #[arg(long)]
    pub log: bool,

    /// Verbose log
    #[arg(short,long)]
    pub verbose: bool,


}

#[derive(Subcommand,Debug)]
pub enum RessourcesCommand{
    /// Ressource list options
    Ressources(RessourcesArgs),
}

#[derive(Debug, Args)]
pub struct RessourcesArgs{
    #[command(subcommand)]
    pub command: ListRessources,
}


#[derive(Debug, Subcommand)]
pub enum ListRessources{
    /// List all target names
    Targets,
    /// List all group name
    Groups,
    /// List all categories and his corresponding ressource name
    Categories,
}

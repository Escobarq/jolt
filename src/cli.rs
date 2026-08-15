use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "jolt")]
#[command(version = "0.1.0")]
#[command(about = "Gestor de paquetes y proyectos Java ultrarrápido", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inicializa un nuevo proyecto Java en el directorio actual o en el indicado
    Init {
        /// Nombre del proyecto a inicializar
        name: Option<String>,
    },
    /// Añade una dependencia al proyecto actual
    Add {
        /// La dependencia en formato groupId:artifactId
        dependency: String,
    },
    /// Resuelve dependencias e instala localmente
    Install,
    /// Compila y empaqueta el proyecto
    Build {
        /// Empaqueta todas las dependencias en un único Fat-JAR autónomo
        #[arg(short, long)]
        standalone: bool,
    },
    /// Ejecuta el proyecto
    Run,
    /// Ejecuta las pruebas unitarias del proyecto con JUnit 5 integrado
    Test,
}

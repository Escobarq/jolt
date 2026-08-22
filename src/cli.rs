use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "jolt")]
#[command(version = "0.2.0")]
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
        /// Plantilla de inicio (minimal, cli, javafx, swing, web, spring)
        #[arg(short, long)]
        template: Option<String>,
        /// Muestra todas las plantillas disponibles
        #[arg(short = 'l', long)]
        list_templates: bool,
    },
    /// Añade una dependencia al proyecto actual
    Add {
        /// La dependencia en formato groupId:artifactId[:version]
        dependency: String,
        /// Añade la dependencia a las dependencias de desarrollo (dev-dependencies)
        #[arg(short = 'D', long = "dev")]
        dev: bool,
    },
    /// Busca dependencias y librerias en Maven Central
    #[command(alias = "find")]
    Search {
        /// Termino de busqueda (nombre de libreria, grupo o descripcion)
        query: String,
        /// Cantidad maxima de resultados a mostrar (por defecto: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Elimina una dependencia del proyecto
    #[command(alias = "rm")]
    Remove {
        /// La dependencia en formato groupId:artifactId
        dependency: String,
    },
    /// Resuelve dependencias e instala localmente
    Install {
        /// Exige que las dependencias coincidan exactamente con jolt.lock (falla si hay discrepancias)
        #[arg(long)]
        locked: bool,
    },
    /// Compila y empaqueta el proyecto
    Build {
        /// Empaqueta todas las dependencias en un único Fat-JAR autónomo
        #[arg(short, long)]
        standalone: bool,
    },
    /// Ejecuta el proyecto
    Run {
        /// Observa cambios en el código fuente y reinicia la aplicación automáticamente (Hot Reload)
        #[arg(short, long)]
        watch: bool,
    },
    /// Ejecuta las pruebas unitarias del proyecto con JUnit 5 integrado
    Test,
    /// Sincroniza dependencias del proyecto y regenera la configuración para VS Code / IDEs
    Sync,
    /// Diagnostica el entorno del sistema (Java, Rust, Caché) y la salud del proyecto actual
    Check,
}


use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "jolt")]
#[command(version)]
#[command(about = "Gestor de paquetes y proyectos Java ultrarrápido", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inicializa un nuevo proyecto Java o un Workspace Monorepo en el directorio actual o en el indicado
    Init {
        /// Nombre del proyecto o módulo a inicializar
        name: Option<String>,
        /// Plantilla de inicio (minimal, cli, javafx, swing, web, spring)
        #[arg(short, long)]
        template: Option<String>,
        /// Muestra todas las plantillas disponibles
        #[arg(short = 'l', long)]
        list_templates: bool,
        /// Paquete raíz / namespace Java (ej: org.equipo.proyecto o com.empresa.app)
        #[arg(short = 'p', long = "package")]
        package: Option<String>,
        /// GroupId de Maven (ej: org.equipo)
        #[arg(short = 'g', long = "group-id")]
        group_id: Option<String>,
        /// Inicializa un workspace / monorepo multimódulo
        #[arg(long = "workspace")]
        workspace: bool,
    },
    /// Añade una dependencia al proyecto actual
    Add {
        /// La dependencia en formato groupId:artifactId[:version]
        dependency: String,
        /// Añade la dependencia a las dependencias de desarrollo (dev-dependencies)
        #[arg(short = 'D', long = "dev")]
        dev: bool,
        /// Módulo del workspace al que añadir la dependencia (si se ejecuta en la raíz)
        #[arg(long = "member", alias = "pkg")]
        member: Option<String>,
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
        /// Módulo del workspace del que remover la dependencia
        #[arg(long = "member", alias = "pkg")]
        member: Option<String>,
    },
    /// Resuelve dependencias e instala localmente
    Install {
        /// Exige que las dependencias coincidan exactamente con jolt.lock (falla si hay discrepancias)
        #[arg(long)]
        locked: bool,
        /// Instala las dependencias de todos los miembros del workspace
        #[arg(long = "all")]
        all: bool,
        /// Módulo específico del workspace a instalar
        #[arg(short = 'p', long = "package", alias = "member")]
        member: Option<String>,
    },
    /// Compila y empaqueta el proyecto (con soporte para Fat-JAR, lanzador nativo o instalador completo)
    Build {
        /// Empaqueta todas las dependencias en un único Fat-JAR autónomo
        #[arg(short, long)]
        standalone: bool,
        /// Empaqueta la aplicación en un lanzador binario nativo (alias de 'jolt package')
        #[arg(long = "package")]
        package: bool,
        /// Genera un instalador nativo del sistema (.msi en Windows, app-image en Linux) mostrando el progreso paso a paso
        #[arg(long = "installer")]
        installer: bool,
        /// Nombre personalizado para el binario o instalador generado
        #[arg(short = 'n', long = "name", alias = "installer-name")]
        name: Option<String>,
        /// Habilita la compresión de ejecutables y librerías con UPX
        #[arg(long = "upx")]
        upx: bool,
        /// Compila a binario nativo autónomo usando GraalVM Native Image
        #[arg(long = "native")]
        native: bool,
        /// Permite descargar automáticamente un JDK (Adoptium Temurin) si no hay uno compatible instalado
        #[arg(long = "download-jdk")]
        download_jdk: bool,
        /// Agrega la aplicación a la variable de entorno PATH del sistema/usuario
        #[arg(long = "add-to-path")]
        add_to_path: bool,
        /// Ámbito de instalación ('per-user' o 'per-machine')
        #[arg(long = "scope")]
        scope: Option<String>,
        /// Compila todos los módulos del workspace
        #[arg(long = "all")]
        all: bool,
        /// Compila un módulo específico del workspace
        #[arg(short = 'p', long = "member", alias = "pkg")]
        member: Option<String>,
    },
    /// Empaqueta la aplicación en un binario nativo o instalador autocontenido (msi, app-image, nsis, exe)
    #[command(alias = "pkg", alias = "bundle")]
    Package {
        /// Tipo de paquete (msi, app-image, nsis, exe). Por defecto: app-image (o msi en Windows)
        #[arg(short = 't', long = "type")]
        r#type: Option<String>,

        /// Directorio de salida para el binario generado (por defecto: dist/)
        #[arg(short = 'o', long = "output", alias = "dest")]
        dest: Option<String>,

        /// Nombre del binario/lanzador nativo (por defecto: nombre del proyecto)
        #[arg(short = 'n', long = "name")]
        name: Option<String>,

        /// Versión de la aplicación (por defecto: versión en jolt.toml)
        #[arg(short = 'v', long = "app-version")]
        app_version: Option<String>,

        /// Clase principal con método main (ej: com.example.Main o Main)
        #[arg(short = 'm', long = "main-class")]
        main_class: Option<String>,

        /// Ruta al archivo de ícono (.png en Linux, .ico en Windows)
        #[arg(short = 'i', long = "icon")]
        icon: Option<String>,

        /// Opciones adicionales para la JVM (ej: "-Xmx512m -Dfile.encoding=UTF-8")
        #[arg(long = "java-options")]
        java_options: Option<String>,

        /// Habilita la compresión de ejecutables y librerías con UPX
        #[arg(long = "upx")]
        upx: bool,

        /// Deshabilita explícitamente la compresión UPX si estaba activa en jolt.toml
        #[arg(long = "no-upx")]
        no_upx: bool,

        /// Agrega la aplicación a la variable de entorno PATH del sistema/usuario
        #[arg(long = "add-to-path")]
        add_to_path: bool,

        /// Ámbito de instalación ('per-user' o 'per-machine')
        #[arg(long = "scope")]
        scope: Option<String>,

        /// Módulo del workspace a empaquetar
        #[arg(short = 'p', long = "member", alias = "pkg")]
        member: Option<String>,

        /// Muestra la salida detallada del proceso
        #[arg(long)]
        verbose: bool,
    },
    /// Ejecuta el proyecto
    Run {
        /// Observa cambios en el código fuente y reinicia la aplicación automáticamente (Hot Reload)
        #[arg(short, long)]
        watch: bool,
        /// Módulo del workspace a ejecutar
        #[arg(short = 'p', long = "member", alias = "pkg")]
        member: Option<String>,
        /// Permite descargar automáticamente un JDK si no hay uno compatible instalado
        #[arg(long = "download-jdk")]
        download_jdk: bool,
    },
    /// Ejecuta las pruebas unitarias del proyecto con JUnit 5 integrado
    Test {
        /// Ejecuta pruebas en todos los módulos del workspace
        #[arg(long = "all")]
        all: bool,
        /// Módulo específico del workspace a ejecutar
        #[arg(short = 'p', long = "member", alias = "pkg")]
        member: Option<String>,
        /// Permite descargar automáticamente un JDK si no hay uno compatible instalado
        #[arg(long = "download-jdk")]
        download_jdk: bool,
    },
    /// Sincroniza dependencias del proyecto y regenera la configuración para VS Code / IDEs
    Sync {
        /// Sincroniza todos los módulos del workspace
        #[arg(long = "all")]
        all: bool,
        /// Sincroniza un módulo específico del workspace
        #[arg(short = 'p', long = "member", alias = "pkg")]
        member: Option<String>,
    },
    /// Diagnostica el entorno del sistema (Java, Rust, Caché) y la salud del proyecto actual
    Check,
}


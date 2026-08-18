import picocli.CommandLine;
import picocli.CommandLine.Command;
import picocli.CommandLine.Option;
import picocli.CommandLine.Parameters;

@Command(name = "cli-app", mixinStandardHelpOptions = true, version = "1.0",
        description = "Aplicacion CLI construida con Picocli y Jolt.")
public class Main implements Runnable {

    @Option(names = {"-u", "--user"}, description = "Nombre del usuario")
    private String user = "Desarrollador";

    @Parameters(paramLabel = "<mensaje>", defaultValue = "Bienvenido a Jolt!", description = "Mensaje a mostrar")
    private String message;

    @Override
    public void run() {
        System.out.println("Hola, " + user + "! -> " + message);
    }

    public static void main(String[] args) {
        int exitCode = new CommandLine(new Main()).execute(args);
        System.exit(exitCode);
    }
}

import io.javalin.Javalin;

public class Main {
    public static void main(String[] args) {
        var app = Javalin.create(/*config*/)
            .get("/", ctx -> ctx.result("Servidor Web en ejecucion con Javalin y Jolt!"))
            .get("/health", ctx -> ctx.json("{\"status\":\"ok\",\"tool\":\"jolt\"}"))
            .start(7070);

        System.out.println("Servidor web iniciado en http://localhost:7070");
    }
}

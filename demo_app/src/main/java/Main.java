import com.google.gson.Gson;
import java.util.Map;

public class Main {
    public static void main(String[] args) {
        Gson gson = new Gson();
        Map<String, Object> data = Map.of(
            "tool", "Jolt",
            "version", "0.1.0",
            "speed", "ultra-fast",
            "message", "¡Java impulsado por Rust y Maven Central!"
        );
        String json = gson.toJson(data);
        System.out.println("Salida JSON generada por Gson: " + json);
    }
}

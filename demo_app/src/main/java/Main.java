import com.google.gson.Gson;
import java.io.InputStream;
import java.util.HashMap;
import java.util.Map;
import java.util.Properties;

public class Main {
    public static void main(String[] args) {
        Gson gson = new Gson();
        Map<String, Object> data = new HashMap<>();
        data.put("tool", "Jolt");
        data.put("mode", "standalone-fat-jar");
        data.put("version", "0.1.0");

        // Leer recurso estático copiado desde src/main/resources/
        try (InputStream is = Main.class.getResourceAsStream("/app.properties")) {
            if (is != null) {
                Properties props = new Properties();
                props.load(is);
                data.put("loaded_resource", props.getProperty("app.description"));
            } else {
                data.put("loaded_resource", "Recurso no encontrado");
            }
        } catch (Exception e) {
            data.put("resource_error", e.getMessage());
        }

        String json = gson.toJson(data);
        System.out.println("🔥 ¡HOT RELOAD FUNCIONANDO AL INSTANTE!: " + json);
    }
}

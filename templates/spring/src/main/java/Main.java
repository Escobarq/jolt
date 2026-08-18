import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

@SpringBootApplication
@RestController
public class Main {

    @GetMapping("/")
    public String index() {
        return "Hola desde Spring Boot con Jolt!";
    }

    @GetMapping("/api/status")
    public String status() {
        return "{\"status\":\"UP\",\"framework\":\"Spring Boot 3\",\"manager\":\"Jolt\"}";
    }

    public static void main(String[] args) {
        SpringApplication.run(Main.class, args);
    }
}

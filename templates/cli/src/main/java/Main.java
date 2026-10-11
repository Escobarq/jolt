import com.williamcallahan.tui4j.Program;
import com.williamcallahan.tui4j.compat.bubbletea.Command;
import com.williamcallahan.tui4j.compat.bubbletea.KeyPressMessage;
import com.williamcallahan.tui4j.compat.bubbletea.Message;
import com.williamcallahan.tui4j.compat.bubbletea.Model;
import com.williamcallahan.tui4j.compat.bubbletea.QuitMessage;
import com.williamcallahan.tui4j.compat.bubbletea.UpdateResult;

public class Main implements Model {
    private final String user;
    private int cursor;
    private static final String[] OPTIONS = {"Saludar", "Ver ayuda", "Salir"};

    public Main(String user) {
        this.user = user;
    }

    @Override
    public Command init() {
        return null;
    }

    @Override
    public UpdateResult<? extends Model> update(Message message) {
        if (message instanceof KeyPressMessage key) {
            return switch (key.key()) {
                case "up", "k" -> UpdateResult.from(move(-1));
                case "down", "j" -> UpdateResult.from(move(1));
                case "enter" -> select();
                case "q", "Q", "esc" -> UpdateResult.from(this, QuitMessage::new);
                default -> UpdateResult.from(this);
            };
        }
        return UpdateResult.from(this);
    }

    private Main move(int delta) {
        cursor = (cursor + delta + OPTIONS.length) % OPTIONS.length;
        return this;
    }

    private UpdateResult<? extends Model> select() {
        if (cursor == 2) {
            return UpdateResult.from(this, QuitMessage::new);
        }
        return UpdateResult.from(this);
    }

    @Override
    public String view() {
        StringBuilder output = new StringBuilder();
        output.append("Jolt TUI4J - Hola, ").append(user).append("!\n\n");
        for (int index = 0; index < OPTIONS.length; index++) {
            output.append(index == cursor ? "> " : "  ").append(OPTIONS[index]).append("\n");
        }
        output.append("\nUsa arriba/abajo o j/k, Enter para seleccionar y q para salir.");
        return output.toString();
    }

    public static void main(String[] args) {
        String user = args.length > 0 ? args[0] : "Desarrollador";
        Main model = new Main(user);
        new Program(model).run();
    }
}

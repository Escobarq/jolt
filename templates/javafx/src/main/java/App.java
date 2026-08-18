import javafx.application.Application;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;

public class App extends Application {
    private int clicks = 0;

    @Override
    public void start(Stage stage) {
        Label title = new Label("Jolt + JavaFX");
        title.setStyle("-fx-font-size: 20px; -fx-font-weight: bold; -fx-text-fill: #333;");

        Label counter = new Label("Clics: 0");
        Button btn = new Button("Presioname");
        btn.setOnAction(e -> {
            clicks++;
            counter.setText("Clics: " + clicks);
        });

        VBox root = new VBox(15, title, btn, counter);
        root.setAlignment(Pos.CENTER);

        Scene scene = new Scene(root, 400, 250);
        stage.setTitle("Jolt JavaFX App");
        stage.setScene(scene);
        stage.show();
    }

    public static void main(String[] args) {
        launch(args);
    }
}

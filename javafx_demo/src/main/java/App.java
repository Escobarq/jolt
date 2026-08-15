import javafx.application.Application;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;

public class App extends Application {
    private int counter = 0;

    @Override
    public void start(Stage stage) {
        Label title = new Label("⚡ Jolt + JavaFX App");
        title.getStyleClass().add("title-label");

        Label subtitle = new Label("¡Construido y gestionado con Jolt!");
        subtitle.getStyleClass().add("subtitle-label");

        Label counterLabel = new Label("Clicks: 0");
        counterLabel.getStyleClass().add("subtitle-label");

        Button button = new Button("¡Haz clic aquí!");
        button.getStyleClass().add("primary-button");
        button.setOnAction(e -> {
            counter++;
            counterLabel.setText("Clicks: " + counter);
        });

        VBox root = new VBox(15, title, subtitle, button, counterLabel);
        root.setAlignment(Pos.CENTER);

        Scene scene = new Scene(root, 450, 300);
        if (getClass().getResource("/style.css") != null) {
            scene.getStylesheets().add(getClass().getResource("/style.css").toExternalForm());
        }

        stage.setTitle("Jolt JavaFX Demo");
        stage.setScene(scene);
        stage.show();
    }

    public static void main(String[] args) {
        launch(args);
    }
}

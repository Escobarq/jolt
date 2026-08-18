import com.formdev.flatlaf.FlatDarkLaf;
import javax.swing.*;
import java.awt.*;

public class Main {
    private static int counter = 0;

    public static void main(String[] args) {
        // Look and Feel moderno oscuro de FlatLaf
        FlatDarkLaf.setup();

        SwingUtilities.invokeLater(() -> {
            JFrame frame = new JFrame("Jolt + Java Swing (FlatLaf)");
            frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
            frame.setSize(450, 280);
            frame.setLocationRelativeTo(null);

            JPanel panel = new JPanel();
            panel.setLayout(new BoxLayout(panel, BoxLayout.Y_AXIS));
            panel.setBorder(BorderFactory.createEmptyBorder(30, 30, 30, 30));

            JLabel title = new JLabel("Aplicacion Java Swing");
            title.setFont(new Font("SansSerif", Font.BOLD, 22));
            title.setAlignmentX(Component.CENTER_ALIGNMENT);

            JLabel counterLabel = new JLabel("Clics realizados: 0");
            counterLabel.setFont(new Font("SansSerif", Font.PLAIN, 16));
            counterLabel.setAlignmentX(Component.CENTER_ALIGNMENT);

            JButton button = new JButton("Incrementar Contador");
            button.setFont(new Font("SansSerif", Font.PLAIN, 14));
            button.setAlignmentX(Component.CENTER_ALIGNMENT);
            button.addActionListener(e -> {
                counter++;
                counterLabel.setText("Clics realizados: " + counter);
            });

            panel.add(title);
            panel.add(Box.createRigidArea(new Dimension(0, 20)));
            panel.add(button);
            panel.add(Box.createRigidArea(new Dimension(0, 15)));
            panel.add(counterLabel);

            frame.add(panel);
            frame.setVisible(true);
        });
    }
}

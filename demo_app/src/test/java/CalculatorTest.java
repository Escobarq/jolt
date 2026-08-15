import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class CalculatorTest {

    @Test
    void testAddition() {
        assertEquals(4, 2 + 2, "La suma debe ser 4");
    }

    @Test
    void testStringValidation() {
        String msg = "Jolt";
        assertTrue(msg.startsWith("J"), "Debe comenzar con J");
    }
}

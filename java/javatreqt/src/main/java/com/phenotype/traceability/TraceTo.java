package com.phenotype.traceability;

import java.lang.annotation.*;
import java.util.regex.Pattern;

/**
 * Annotation to mark a test as tracing to a Feature Requirement (FR).
 *
 * <p>Example usage:
 * <pre>{@code
 * @Test
 * @TraceTo({"FR-EXAMPLE-001"})
 * public void testFeature() {
 *     // test code
 * }
 * }</pre>
 */
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.METHOD)
@Repeatable(TraceTo.Container.class)
public @interface TraceTo {
    String[] value();

    /**
     * Container for repeated @TraceTo annotations.
     */
    @Retention(RetentionPolicy.RUNTIME)
    @Target(ElementType.METHOD)
    @interface Container {
        TraceTo[] value();
    }

    /**
     * Validates an FR ID format.
     */
    class Validator {
        private static final Pattern FR_PATTERN = Pattern.compile("^FR-[A-Z][A-Z0-9]*-\\d{3,}(-[A-Z0-9]+)?$");

        public static boolean validate(String frId) {
            return FR_PATTERN.matcher(frId).matches();
        }
    }
}

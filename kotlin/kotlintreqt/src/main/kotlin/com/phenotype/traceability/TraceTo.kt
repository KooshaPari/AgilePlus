package com.phenotype.traceability

/**
 * Annotation to mark a test as tracing to a Feature Requirement (FR).
 *
 * Example usage:
 * ```kotlin
 * @Test
 * @TraceTo("FR-EXAMPLE-001")
 * fun testFeature() {
 *     // test code
 * }
 * ```
 *
 * @property value One or more FR IDs (e.g., "FR-EXAMPLE-001")
 */
@Retention(AnnotationRetention.RUNTIME)
@Target(AnnotationTarget.FUNCTION)
annotation class TraceTo(vararg val value: String)

/**
 * Validates an FR ID format: FR-XXXX-NNN or FR-XXXX-NNN-YYY
 */
fun validateFrId(frId: String): Boolean {
    val pattern = Regex("^FR-[A-Z][A-Z0-9]*-\\d{3,}(-[A-Z0-9]+)?$")
    return pattern.matches(frId)
}

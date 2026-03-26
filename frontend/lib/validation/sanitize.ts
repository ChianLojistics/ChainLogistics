const HTML_ENTITIES: Record<string, string> = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#x27;",
  "/": "&#x2F;",
};

const HTML_ENTITY_PATTERN = /[&<>"'/]/g;

/**
 * Escapes HTML special characters to prevent XSS in rendered output.
 */
export function escapeHtml(input: string): string {
  return input.replace(HTML_ENTITY_PATTERN, (char) => HTML_ENTITIES[char] ?? char);
}

/**
 * Strips all HTML tags from a string.
 */
export function stripHtmlTags(input: string): string {
  return input.replace(/<[^>]*>/g, "");
}

/**
 * Sanitizes a user-provided string by stripping HTML tags and trimming whitespace.
 * Use this on all free-text inputs before submitting to contracts or APIs.
 */
export function sanitizeInput(input: string): string {
  return stripHtmlTags(input).trim();
}

/**
 * Sanitizes all string values in a record. Non-string values are passed through.
 */
export function sanitizeFormData<T extends Record<string, unknown>>(data: T): T {
  const result = { ...data };
  for (const key of Object.keys(result)) {
    const value = result[key];
    if (typeof value === "string") {
      (result as Record<string, unknown>)[key] = sanitizeInput(value);
    }
  }
  return result;
}

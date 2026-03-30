import { toast } from "@/lib/toast/store";
import { classifyError, type ClassifiedError } from "@/lib/errors";

export type SuccessAction = {
  label: string;
  onClick: () => void;
};

export function showSuccess(message: string, action?: SuccessAction): string {
  return toast.success(message, {
    duration: 6000,
    action: action ? { label: action.label, onClick: action.onClick } : undefined,
  });
}

export function showErrorFeedback(error: unknown): ClassifiedError {
  const classified = classifyError(error);

  const recoveryText =
    classified.recoverySteps.length > 0
      ? `\n${classified.recoverySteps[0]}`
      : "";

  toast.error(`${classified.message}${recoveryText}`, {
    title: classified.title,
    duration: classified.retryable ? 8000 : 10000,
  });

  return classified;
}

export function showWarning(message: string): string {
  return toast.warning(message, { duration: 5000 });
}

export function showInfo(message: string): string {
  return toast.info(message, { duration: 4000 });
}

export type RetryOptions = {
  maxAttempts?: number;
  baseDelayMs?: number;
  maxDelayMs?: number;
  onAttempt?: (attempt: number, maxAttempts: number) => void;
};

export async function withRetry<T>(
  fn: () => Promise<T>,
  options?: RetryOptions
): Promise<T> {
  const maxAttempts = options?.maxAttempts ?? 3;
  const baseDelayMs = options?.baseDelayMs ?? 1000;
  const maxDelayMs = options?.maxDelayMs ?? 10000;

  let lastError: unknown;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      options?.onAttempt?.(attempt, maxAttempts);
      return await fn();
    } catch (error) {
      lastError = error;

      if (attempt === maxAttempts) break;

      const delay = Math.min(baseDelayMs * Math.pow(2, attempt - 1), maxDelayMs);
      const jitter = delay * 0.1 * Math.random();
      await new Promise((resolve) => setTimeout(resolve, delay + jitter));
    }
  }

  throw lastError;
}

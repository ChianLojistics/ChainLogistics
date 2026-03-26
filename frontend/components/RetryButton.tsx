"use client"

import * as React from "react"
import { RefreshCw } from "lucide-react"

import { cn } from "@/lib/utils"
import { Button, type ButtonProps } from "@/components/ui/button"

export type RetryButtonProps = Omit<ButtonProps, "onClick"> & {
  onRetry: () => Promise<void> | void
  maxAttempts?: number
  baseDelayMs?: number
  maxDelayMs?: number
  label?: string
  retryingLabel?: string
}

export function RetryButton({
  onRetry,
  maxAttempts = 3,
  baseDelayMs = 1000,
  maxDelayMs = 10000,
  label = "Retry",
  retryingLabel = "Retrying...",
  className,
  disabled,
  ...props
}: RetryButtonProps) {
  const [attempt, setAttempt] = React.useState(0)
  const [retrying, setRetrying] = React.useState(false)

  const exhausted = attempt >= maxAttempts

  const handleClick = React.useCallback(async () => {
    if (retrying || exhausted) return

    setRetrying(true)
    const currentAttempt = attempt + 1
    setAttempt(currentAttempt)

    if (currentAttempt > 1) {
      const delay = Math.min(
        baseDelayMs * Math.pow(2, currentAttempt - 2),
        maxDelayMs
      )
      const jitter = delay * 0.1 * Math.random()
      await new Promise((resolve) => setTimeout(resolve, delay + jitter))
    }

    try {
      await onRetry()
    } finally {
      setRetrying(false)
    }
  }, [onRetry, retrying, exhausted, attempt, baseDelayMs, maxDelayMs])

  const reset = React.useCallback(() => {
    setAttempt(0)
    setRetrying(false)
  }, [])

  React.useEffect(() => {
    return reset
  }, [reset])

  const buttonLabel = retrying
    ? retryingLabel
    : exhausted
      ? "No more retries"
      : attempt > 0
        ? `${label} (${maxAttempts - attempt} left)`
        : label

  return (
    <Button
      type="button"
      variant="outline"
      onClick={handleClick}
      disabled={disabled || retrying || exhausted}
      className={cn("gap-2", className)}
      {...props}
    >
      <RefreshCw
        className={cn("h-4 w-4", retrying && "animate-spin")}
      />
      {buttonLabel}
    </Button>
  )
}

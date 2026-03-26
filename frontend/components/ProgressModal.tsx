"use client"

import * as React from "react"
import { Loader2, CheckCircle2, XCircle } from "lucide-react"

import { cn } from "@/lib/utils"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"

export type ProgressStep = {
  label: string
  status: "pending" | "active" | "complete" | "error"
}

export type ProgressModalProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  description?: string
  steps: ProgressStep[]
  errorMessage?: string
  onRetry?: () => void
  onCancel?: () => void
}

function StepIcon({ status }: { status: ProgressStep["status"] }) {
  if (status === "complete") {
    return <CheckCircle2 className="h-4 w-4 text-emerald-600" />
  }
  if (status === "error") {
    return <XCircle className="h-4 w-4 text-destructive" />
  }
  if (status === "active") {
    return <Loader2 className="h-4 w-4 animate-spin text-blue-600" />
  }
  return (
    <div className="h-4 w-4 rounded-full border-2 border-muted-foreground/30" />
  )
}

export function ProgressModal({
  open,
  onOpenChange,
  title,
  description,
  steps,
  errorMessage,
  onRetry,
  onCancel,
}: ProgressModalProps) {
  const hasError = steps.some((s) => s.status === "error")
  const isComplete = steps.length > 0 && steps.every((s) => s.status === "complete")
  const isPending = steps.some((s) => s.status === "active")
  const canClose = !isPending

  const completedCount = steps.filter((s) => s.status === "complete").length
  const progressPercent =
    steps.length > 0 ? Math.round((completedCount / steps.length) * 100) : 0

  return (
    <Dialog open={open} onOpenChange={(next) => (canClose ? onOpenChange(next) : undefined)}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          {description && <DialogDescription>{description}</DialogDescription>}
        </DialogHeader>

        <div className="mt-2 space-y-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="font-medium">Progress</span>
              <span className="text-muted-foreground">{progressPercent}%</span>
            </div>
            <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
              <div
                className={cn(
                  "h-full rounded-full transition-all duration-500",
                  hasError
                    ? "bg-destructive"
                    : isComplete
                      ? "bg-emerald-500"
                      : "bg-blue-600",
                  isPending && "animate-pulse"
                )}
                style={{ width: `${progressPercent}%` }}
              />
            </div>
          </div>

          <div className="space-y-2">
            {steps.map((step, idx) => (
              <div key={idx} className="flex items-center gap-3">
                <StepIcon status={step.status} />
                <span
                  className={cn(
                    "text-sm",
                    step.status === "pending" && "text-muted-foreground",
                    step.status === "active" && "font-medium text-foreground",
                    step.status === "complete" && "text-foreground",
                    step.status === "error" && "text-destructive"
                  )}
                >
                  {step.label}
                </span>
              </div>
            ))}
          </div>

          {hasError && errorMessage && (
            <div className="rounded-lg border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive">
              {errorMessage}
            </div>
          )}

          {isComplete && (
            <div className="rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-3 text-sm text-emerald-700">
              All steps completed successfully.
            </div>
          )}
        </div>

        <DialogFooter className="mt-4">
          {hasError && onRetry && (
            <Button onClick={onRetry}>Retry</Button>
          )}
          {isPending && onCancel && (
            <Button variant="secondary" onClick={onCancel}>
              Cancel
            </Button>
          )}
          {canClose && (
            <Button
              variant={isComplete ? "default" : "secondary"}
              onClick={() => onOpenChange(false)}
            >
              {isComplete ? "Done" : "Close"}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

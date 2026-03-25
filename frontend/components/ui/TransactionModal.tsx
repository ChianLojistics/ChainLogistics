"use client"

import * as React from "react"
import { CheckCircle2, ExternalLink, Loader2, PenLine, XCircle } from "lucide-react"

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

export type TransactionStep = "preparing" | "signing" | "submitting" | "complete" | "error"

export type TransactionModalProps = {
  open: boolean
  onOpenChange: (open: boolean) => void

  title?: string
  description?: string

  step: TransactionStep
  errorTitle?: string
  errorMessage?: string

  txHash?: string | null
  network?: "testnet" | "mainnet" | "futurenet"

  onRetry?: () => void
  retryLabel?: string

  closeLabel?: string
  allowCloseWhilePending?: boolean

  showConfetti?: boolean
}

const STEP_ORDER: Array<Exclude<TransactionStep, "error">> = [
  "preparing",
  "signing",
  "submitting",
  "complete",
]

const STEP_LABEL: Record<Exclude<TransactionStep, "error">, string> = {
  preparing: "Preparing",
  signing: "Signing",
  submitting: "Submitting",
  complete: "Complete",
}

function stepToProgress(step: TransactionStep): number {
  if (step === "error") return 0
  const idx = STEP_ORDER.indexOf(step)
  if (idx < 0) return 0
  return Math.round(((idx + 1) / STEP_ORDER.length) * 100)
}

function explorerUrlForTxHash(txHash: string, network?: TransactionModalProps["network"]): string {
  const net = network ?? "testnet"
  // Horizon transaction endpoint is a reliable explorer link.
  const base =
    net === "mainnet"
      ? "https://horizon.stellar.org"
      : net === "futurenet"
        ? "https://horizon-futurenet.stellar.org"
        : "https://horizon-testnet.stellar.org"

  return `${base}/transactions/${encodeURIComponent(txHash)}`
}

function Confetti({ active }: { active: boolean }) {
  if (!active) return null

  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden">
      <div className="absolute inset-x-0 -top-6 mx-auto h-1 w-1">
        {Array.from({ length: 24 }).map((_, i) => {
          const left = (i / 24) * 100
          const delay = (i % 6) * 0.06
          const duration = 1.1 + (i % 7) * 0.08
          const size = 6 + (i % 5) * 2
          const colors = [
            "bg-emerald-500",
            "bg-sky-500",
            "bg-fuchsia-500",
            "bg-amber-400",
            "bg-violet-500",
            "bg-rose-500",
          ]
          const color = colors[i % colors.length]

          return (
            <span
              key={i}
              className={cn(
                "absolute block rounded-sm opacity-90",
                color,
                "animate-[txconfetti_1.4s_ease-in_forwards]"
              )}
              style={{
                left: `${left}%`,
                width: `${size}px`,
                height: `${size * 0.6}px`,
                animationDelay: `${delay}s`,
                animationDuration: `${duration}s`,
                transform: `translateX(-50%) rotate(${(i % 12) * 15}deg)`,
              }}
            />
          )
        })}
      </div>
      <style jsx>{`
        @keyframes txconfetti {
          0% {
            transform: translate(-50%, 0) rotate(0deg);
            opacity: 1;
          }
          100% {
            transform: translate(-50%, 260px) rotate(360deg);
            opacity: 0;
          }
        }
      `}</style>
    </div>
  )
}

function StepRow({
  label,
  state,
}: {
  label: string
  state: "done" | "active" | "todo" | "error"
}) {
  const Icon =
    state === "done" ? CheckCircle2 : state === "error" ? XCircle : state === "active" ? Loader2 : PenLine

  return (
    <div className="flex items-center gap-3">
      <span
        className={cn(
          "flex h-7 w-7 items-center justify-center rounded-full border",
          state === "done" && "border-emerald-500/30 bg-emerald-500/10 text-emerald-600",
          state === "active" && "border-blue-500/30 bg-blue-500/10 text-blue-600",
          state === "todo" && "border-muted-foreground/20 bg-muted/30 text-muted-foreground",
          state === "error" && "border-destructive/30 bg-destructive/10 text-destructive"
        )}
      >
        <Icon className={cn("h-4 w-4", state === "active" && "animate-spin")} />
      </span>
      <span
        className={cn(
          "text-sm font-medium",
          state === "todo" && "text-muted-foreground",
          state === "active" && "text-foreground",
          state === "done" && "text-foreground",
          state === "error" && "text-destructive"
        )}
      >
        {label}
      </span>
    </div>
  )
}

export function TransactionModal({
  open,
  onOpenChange,
  title = "Transaction",
  description = "Follow the transaction progress.",
  step,
  errorTitle = "Transaction failed",
  errorMessage,
  txHash,
  network,
  onRetry,
  retryLabel = "Retry",
  closeLabel = "Close",
  allowCloseWhilePending = false,
  showConfetti = false,
}: TransactionModalProps) {
  const pending = step === "preparing" || step === "signing" || step === "submitting"
  const canClose = !pending || allowCloseWhilePending
  const progress = stepToProgress(step)

  const explorerHref = txHash ? explorerUrlForTxHash(txHash, network) : null

  const stepState = (s: Exclude<TransactionStep, "error">): "done" | "active" | "todo" | "error" => {
    if (step === "error") return "error"
    const activeIdx = STEP_ORDER.indexOf(step)
    const idx = STEP_ORDER.indexOf(s)
    if (idx < activeIdx) return "done"
    if (idx === activeIdx) return step === "complete" ? "done" : "active"
    return "todo"
  }

  return (
    <Dialog open={open} onOpenChange={(next) => (canClose ? onOpenChange(next) : undefined)}>
      <DialogContent className="overflow-hidden">
        <div className="relative">
          <Confetti active={showConfetti && step === "complete"} />
          <DialogHeader>
            <DialogTitle>{title}</DialogTitle>
            <DialogDescription>{description}</DialogDescription>
          </DialogHeader>

          <div className="mt-4 space-y-4">
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Progress</span>
                <span className="text-sm text-muted-foreground">{progress}%</span>
              </div>
              <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
                <div
                  className={cn(
                    "h-full rounded-full transition-all duration-500",
                    step === "error" ? "bg-destructive" : step === "complete" ? "bg-emerald-500" : "bg-blue-600",
                    pending && "animate-pulse"
                  )}
                  style={{ width: `${progress}%` }}
                />
              </div>
            </div>

            <div className="grid gap-2">
              <StepRow label={STEP_LABEL.preparing} state={stepState("preparing")} />
              <StepRow label={STEP_LABEL.signing} state={stepState("signing")} />
              <StepRow label={STEP_LABEL.submitting} state={stepState("submitting")} />
              <StepRow label={STEP_LABEL.complete} state={stepState("complete")} />
            </div>

            {step === "complete" && (
              <div className="rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-3 text-sm text-emerald-700">
                <div className="font-medium">Transaction confirmed</div>
                {explorerHref && (
                  <a
                    href={explorerHref}
                    target="_blank"
                    rel="noreferrer"
                    className="mt-1 inline-flex items-center gap-1 text-emerald-700 underline underline-offset-2 hover:opacity-80"
                  >
                    View on explorer
                    <ExternalLink className="h-4 w-4" />
                  </a>
                )}
                {!explorerHref && txHash && (
                  <div className="mt-1 break-all text-xs text-emerald-700/80">{txHash}</div>
                )}
              </div>
            )}

            {step === "error" && (
              <div className="rounded-lg border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive">
                <div className="font-medium">{errorTitle}</div>
                {errorMessage && <div className="mt-1 whitespace-pre-wrap text-xs">{errorMessage}</div>}
                {explorerHref && (
                  <a
                    href={explorerHref}
                    target="_blank"
                    rel="noreferrer"
                    className="mt-2 inline-flex items-center gap-1 underline underline-offset-2 hover:opacity-80"
                  >
                    View attempted transaction
                    <ExternalLink className="h-4 w-4" />
                  </a>
                )}
              </div>
            )}

            {(pending || step === "error") && (
              <div className="rounded-lg bg-muted/40 p-3 text-xs text-muted-foreground">
                {pending
                  ? "Keep this window open while your wallet signs and the transaction is submitted."
                  : "You can retry the transaction or close this dialog."}
              </div>
            )}
          </div>

          <DialogFooter className="mt-6">
            {step === "error" && onRetry && (
              <Button onClick={onRetry}>
                {retryLabel}
              </Button>
            )}
            <Button
              variant={step === "complete" ? "default" : "secondary"}
              onClick={() => (canClose ? onOpenChange(false) : undefined)}
              disabled={!canClose}
            >
              {closeLabel}
            </Button>
          </DialogFooter>
        </div>
      </DialogContent>
    </Dialog>
  )
}

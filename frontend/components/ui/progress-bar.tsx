import { cn } from "@/lib/utils";

interface ProgressBarProps extends React.HTMLAttributes<HTMLDivElement> {
  /**
   * Progress percentage (0-100)
   */
  value: number;
  /**
   * Maximum value (default: 100)
   */
  max?: number;
  /**
   * Show percentage label
   */
  showLabel?: boolean;
  /**
   * Show indeterminate state
   */
  indeterminate?: boolean;
  /**
   * Color variant
   */
  variant?: "default" | "success" | "warning" | "error";
  /**
   * Size variant
   */
  size?: "sm" | "md" | "lg";
}

const variantClasses = {
  default: "bg-gradient-to-r from-blue-500 to-blue-600",
  success: "bg-gradient-to-r from-green-500 to-green-600",
  warning: "bg-gradient-to-r from-yellow-500 to-yellow-600",
  error: "bg-gradient-to-r from-red-500 to-red-600",
};

const sizeClasses = {
  sm: "h-1",
  md: "h-2",
  lg: "h-3",
};

export function ProgressBar({
  value,
  max = 100,
  showLabel = false,
  indeterminate = false,
  variant = "default",
  size = "md",
  className,
  ...props
}: ProgressBarProps) {
  const percentage = Math.min((value / max) * 100, 100);

  return (
    <div
      className={cn("w-full", className)}
      role="progressbar"
      aria-valuenow={value}
      aria-valuemin={0}
      aria-valuemax={max}
      aria-label="Loading progress"
      {...props}
    >
      <div
        className={cn(
          "w-full bg-gray-200 rounded-full overflow-hidden",
          sizeClasses[size]
        )}
      >
        <div
          className={cn(
            "h-full transition-all duration-300 rounded-full",
            variantClasses[variant],
            indeterminate && "animate-pulse"
          )}
          style={{
            width: indeterminate ? "100%" : `${percentage}%`,
          }}
        />
      </div>
      {showLabel && (
        <div className="mt-1 text-sm text-gray-600 text-center">
          {Math.round(percentage)}%
        </div>
      )}
    </div>
  );
}

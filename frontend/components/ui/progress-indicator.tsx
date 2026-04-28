import { cn } from "@/lib/utils";
import { Check, Clock } from "lucide-react";

interface ProgressStep {
  id: string;
  label: string;
  description?: string;
}

interface ProgressIndicatorProps {
  /**
   * Array of steps
   */
  steps: ProgressStep[];
  /**
   * Current step index (0-based)
   */
  currentStep: number;
  /**
   * Completed step indices
   */
  completedSteps?: number[];
  /**
   * Orientation (vertical or horizontal)
   */
  orientation?: "horizontal" | "vertical";
  /**
   * Show step descriptions
   */
  showDescriptions?: boolean;
  /**
   * Custom className
   */
  className?: string;
}

export function ProgressIndicator({
  steps,
  currentStep,
  completedSteps = [],
  orientation = "horizontal",
  showDescriptions = true,
  className,
}: ProgressIndicatorProps) {
  return (
    <div
      className={cn(
        "w-full",
        orientation === "vertical" ? "space-y-6" : "space-y-4",
        className
      )}
      role="progressbar"
      aria-valuenow={currentStep + 1}
      aria-valuemin={1}
      aria-valuemax={steps.length}
      aria-label={`Progress: Step ${currentStep + 1} of ${steps.length}`}
    >
      {orientation === "horizontal" ? (
        <div className="flex items-center justify-between">
          {steps.map((step, index) => (
            <div key={step.id} className="flex flex-col items-center flex-1">
              {/* Step Circle */}
              <div
                className={cn(
                  "w-10 h-10 rounded-full flex items-center justify-center font-semibold transition-all",
                  completedSteps.includes(index)
                    ? "bg-green-500 text-white"
                    : index === currentStep
                      ? "bg-blue-500 text-white ring-2 ring-blue-300"
                      : "bg-gray-200 text-gray-600"
                )}
              >
                {completedSteps.includes(index) ? (
                  <Check size={20} className="aria-label=Completed" />
                ) : (
                  index + 1
                )}
              </div>

              {/* Step Label */}
              <div className="mt-2 text-center">
                <p
                  className={cn(
                    "text-sm font-medium",
                    index === currentStep
                      ? "text-blue-600"
                      : completedSteps.includes(index)
                        ? "text-green-600"
                        : "text-gray-600"
                  )}
                >
                  {step.label}
                </p>
              </div>

              {/* Connector Line */}
              {index < steps.length - 1 && (
                <div
                  className={cn(
                    "h-1 mt-4 flex-1 mx-2",
                    completedSteps.includes(index)
                      ? "bg-green-500"
                      : "bg-gray-200"
                  )}
                  role="separator"
                />
              )}
            </div>
          ))}
        </div>
      ) : (
        <div className="space-y-6">
          {steps.map((step, index) => (
            <div key={step.id} className="flex gap-4">
              {/* Vertical Step Circle */}
              <div className="flex flex-col items-center">
                <div
                  className={cn(
                    "w-10 h-10 rounded-full flex items-center justify-center font-semibold transition-all",
                    completedSteps.includes(index)
                      ? "bg-green-500 text-white"
                      : index === currentStep
                        ? "bg-blue-500 text-white ring-2 ring-blue-300"
                        : "bg-gray-200 text-gray-600"
                  )}
                >
                  {completedSteps.includes(index) ? (
                    <Check size={20} />
                  ) : (
                    index + 1
                  )}
                </div>

                {/* Vertical Connector */}
                {index < steps.length - 1 && (
                  <div
                    className={cn(
                      "w-1 h-12 mt-2",
                      completedSteps.includes(index)
                        ? "bg-green-500"
                        : "bg-gray-200"
                    )}
                  />
                )}
              </div>

              {/* Step Content */}
              <div className="pt-1">
                <p
                  className={cn(
                    "text-sm font-medium",
                    index === currentStep
                      ? "text-blue-600"
                      : completedSteps.includes(index)
                        ? "text-green-600"
                        : "text-gray-600"
                  )}
                >
                  {step.label}
                </p>
                {showDescriptions && step.description && (
                  <p className="text-xs text-gray-500 mt-1">
                    {step.description}
                  </p>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

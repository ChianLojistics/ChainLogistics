import * as React from "react";

import { cn } from "@/lib/utils";

export interface StatCardProps {
  label: string;
  value: React.ReactNode;
  description?: string;
  icon?: React.ReactNode;
  className?: string;
}

export function StatCard({
  label,
  value,
  description,
  icon,
  className,
}: Readonly<StatCardProps>) {
  return (
    <div
      className={cn(
        "rounded-xl border border-zinc-200 bg-white p-5 shadow-sm",
        className
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="text-sm font-medium text-zinc-600">{label}</p>
          <div className="mt-1 text-2xl font-semibold text-zinc-900">
            {value}
          </div>
          {description ? (
            <p className="mt-1 text-sm text-zinc-500">{description}</p>
          ) : null}
        </div>
        {icon ? (
          <div className="shrink-0 text-zinc-400" aria-hidden="true">
            {icon}
          </div>
        ) : null}
      </div>
    </div>
  );
}

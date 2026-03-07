import * as React from "react";
import {
  BadgeCheck,
  ClipboardPen,
  Cog,
  Inbox,
  Leaf,
  MapPin,
  Package,
  RefreshCw,
  Truck,
} from "lucide-react";

import { cn } from "@/lib/utils";

export type EventTypeBadgeSize = "sm" | "md" | "lg";

const EVENT_TYPE_META = {
  HARVEST: {
    label: "Harvest",
    className: "bg-green-50 text-green-800 border-green-200",
    icon: Leaf,
  },
  PROCESS: {
    label: "Process",
    className: "bg-blue-50 text-blue-800 border-blue-200",
    icon: Cog,
  },
  PACKAGE: {
    label: "Package",
    className: "bg-purple-50 text-purple-800 border-purple-200",
    icon: Package,
  },
  SHIP: {
    label: "Ship",
    className: "bg-orange-50 text-orange-800 border-orange-200",
    icon: Truck,
  },
  RECEIVE: {
    label: "Receive",
    className: "bg-cyan-50 text-cyan-800 border-cyan-200",
    icon: Inbox,
  },
  QUALITY_CHECK: {
    label: "Quality Check",
    className: "bg-yellow-50 text-yellow-900 border-yellow-200",
    icon: BadgeCheck,
  },
  TRANSFER: {
    label: "Transfer",
    className: "bg-indigo-50 text-indigo-800 border-indigo-200",
    icon: RefreshCw,
  },
  REGISTER: {
    label: "Register",
    className: "bg-gray-50 text-gray-800 border-gray-200",
    icon: ClipboardPen,
  },
  CHECKPOINT: {
    label: "Checkpoint",
    className: "bg-pink-50 text-pink-800 border-pink-200",
    icon: MapPin,
  },
} satisfies Record<
  string,
  {
    label: string;
    className: string;
    icon: React.ComponentType<React.SVGProps<SVGSVGElement>>;
  }
>;

export function getEventTypeBadgeMeta(eventType: string) {
  return (
    EVENT_TYPE_META[eventType as keyof typeof EVENT_TYPE_META] ||
    EVENT_TYPE_META.REGISTER
  );
}

const SIZE_STYLES: Record<
  EventTypeBadgeSize,
  { container: string; icon: string; label: string }
> = {
  sm: {
    container: "h-5 px-2 text-[11px]",
    icon: "h-3.5 w-3.5",
    label: "leading-4",
  },
  md: {
    container: "h-6 px-2.5 text-xs",
    icon: "h-4 w-4",
    label: "leading-5",
  },
  lg: {
    container: "h-7 px-3 text-sm",
    icon: "h-5 w-5",
    label: "leading-6",
  },
};

export interface EventTypeBadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  eventType: string;
  size?: EventTypeBadgeSize;
  label?: string;
}

export function EventTypeBadge({
  eventType,
  size = "md",
  label,
  className,
  ...props
}: EventTypeBadgeProps) {
  const meta = getEventTypeBadgeMeta(eventType);

  const Icon = meta.icon;
  const resolvedLabel = label ?? meta.label;
  const sizeStyles = SIZE_STYLES[size];

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full border font-semibold",
        meta.className,
        sizeStyles.container,
        className
      )}
      {...props}
    >
      <Icon className={cn("shrink-0", sizeStyles.icon)} aria-hidden="true" />
      <span className={cn("whitespace-nowrap", sizeStyles.label)}>{resolvedLabel}</span>
    </span>
  );
}

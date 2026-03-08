import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

const emptyStateVariants = cva(
  "w-full flex flex-col items-center justify-center text-center",
  {
    variants: {
      variant: {
        default: "rounded-xl border border-dashed bg-muted/20",
        card: "rounded-xl border bg-card",
        subtle: "rounded-xl",
      },
      size: {
        sm: "px-4 py-8",
        md: "px-6 py-12",
        lg: "px-8 py-16",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "md",
    },
  }
);

export type EmptyStateCta = {
  label: string;
  onClick?: () => void;
  href?: string;
  target?: React.HTMLAttributeAnchorTarget;
  rel?: string;
  variant?: React.ComponentProps<typeof Button>["variant"];
  size?: React.ComponentProps<typeof Button>["size"];
  disabled?: boolean;
};

export interface EmptyStateProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof emptyStateVariants> {
  illustration?: React.ReactNode;
  icon?: React.ReactNode;
  title: React.ReactNode;
  description?: React.ReactNode;
  cta?: EmptyStateCta;
}

export function EmptyState({
  className,
  variant,
  size,
  illustration,
  icon,
  title,
  description,
  cta,
  ...props
}: Readonly<EmptyStateProps>) {
  const visual = illustration ?? icon;

  return (
    <div
      className={cn(emptyStateVariants({ variant, size }), className)}
      {...props}
    >
      {visual ? (
        <div className="mb-4 flex items-center justify-center">
          <div className="text-muted-foreground [&_svg]:h-12 [&_svg]:w-12">
            {visual}
          </div>
        </div>
      ) : null}

      <div className="max-w-[42rem]">
        <div className="text-base font-semibold leading-tight tracking-tight">
          {title}
        </div>
        {description ? (
          <div className="mt-2 text-sm text-muted-foreground">
            {description}
          </div>
        ) : null}

        {cta ? (
          <div className="mt-5 flex items-center justify-center">
            {cta.href ? (
              <Button
                asChild
                variant={cta.variant ?? "default"}
                size={cta.size ?? "default"}
                disabled={cta.disabled}
              >
                <a
                  href={cta.href}
                  target={cta.target}
                  rel={cta.rel ?? (cta.target === "_blank" ? "noreferrer" : undefined)}
                >
                  {cta.label}
                </a>
              </Button>
            ) : (
              <Button
                type="button"
                onClick={cta.onClick}
                variant={cta.variant ?? "default"}
                size={cta.size ?? "default"}
                disabled={cta.disabled}
              >
                {cta.label}
              </Button>
            )}
          </div>
        ) : null}
      </div>
    </div>
  );
}
